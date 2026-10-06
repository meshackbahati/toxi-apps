//! Link endpoints. Validation first, then the link service.

use toxi::json_response;
use toxi::prelude::*;
use toxi_core::extract::PathParams;
use toxi_core::request::RequestExt;
use http_body_util::BodyExt;
use std::sync::Arc;

use crate::models::CreateLink;
use crate::services::links;
use crate::validators;
use crate::AppState;

fn code_of(req: &Request) -> String {
    req.extensions()
        .get::<PathParams>()
        .and_then(|p| p.0.get("code"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// POST /links — create a short link.
pub async fn create(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body: CreateLink = req.json().await?;
    validators::url(&body.url)?;
    let wanted = body.code.unwrap_or_default();
    validators::custom(&wanted)?;
    let code = links::insert(&state.db, &state.cache, &body.url, &wanted).await?;
    Ok(json_response!({ "code": code, "url": body.url }))
}

/// GET /:code — redirect to the target, rate-limited per caller.
pub async fn redirect(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let code = code_of(&req);
    validators::code(&code)?;

    let caller = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    if !state.limiter.check(caller, "redirect").await {
        return Err(Error::RateLimited("slow down".to_string()));
    }

    match links::resolve(&state.db, &state.cache, &code).await? {
        None => Err(Error::NotFound("unknown code".to_string())),
        Some(url) => {
            links::bump(&state.db, &code).await;
            redirect_to(&url)
        }
    }
}

fn redirect_to(url: &str) -> Result<Response> {
    let res = http::Response::builder()
        .status(http::StatusCode::FOUND)
        .header(http::header::LOCATION, url)
        .body(http_body_util::Full::new(bytes::Bytes::new()).map_err(|e| match e {}).boxed())
        .map_err(|e| Error::InternalServerError(format!("build: {e}")))?;
    Ok(toxi_core::ToxiResponse::new(res))
}

/// GET /links — newest first.
pub async fn list(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    Ok(json_response!({ "links": links::list(&state.db).await? }))
}

/// GET /stats — totals plus cache hits.
pub async fn stats(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let (n, h) = links::counts(&state.db).await?;
    let cached = state.cache.stats();
    Ok(json_response!({ "links": n, "hits": h, "cached": cached.hits }))
}
