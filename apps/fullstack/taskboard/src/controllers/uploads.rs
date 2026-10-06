//! Upload endpoints. Filenames validate, bytes cap at 5 MB.

use toxi::json_response;
use toxi::prelude::*;
use toxi_core::extract::PathParams;
use http_body_util::BodyExt;
use std::sync::Arc;

use crate::services::uploads;
use crate::AppState;

/// POST /uploads?name=... — save the raw body as a file.
pub async fn upload(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let _ = state;
    let query = req.uri().query().unwrap_or("").to_string();
    let raw = query
        .split('&')
        .find_map(|pair| pair.strip_prefix("name="))
        .unwrap_or("file");
    let bytes = req
        .body_mut()
        .collect()
        .await
        .map_err(|e| Error::InternalServerError(format!("body read: {e}")))?
        .to_bytes();
    let stored = uploads::save(&bytes, raw).await?;
    Ok(json_response!({ "file": stored, "bytes": bytes.len() }))
}

/// GET /uploads/:name — serve a stored file.
pub async fn download(req: Request) -> Result<Response> {
    let name = req
        .extensions()
        .get::<PathParams>()
        .and_then(|p| p.0.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let data = uploads::read(&name).await?;
    let res = http::Response::builder()
        .header(http::header::CONTENT_TYPE, "application/octet-stream")
        .header(http::header::CONTENT_LENGTH, data.len())
        .body(http_body_util::Full::new(bytes::Bytes::from(data)).map_err(|e| match e {}).boxed())
        .map_err(|e| Error::InternalServerError(format!("build: {e}")))?;
    Ok(toxi_core::ToxiResponse::new(res))
}
