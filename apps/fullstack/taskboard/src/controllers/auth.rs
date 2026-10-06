//! Auth endpoints. Validation first, then the auth service.

use toxi::json_response;
use toxi::prelude::*;
use toxi_core::request::RequestExt;
use std::sync::Arc;

use crate::services::auth;
use crate::validators;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

/// POST /auth/register — create a user.
pub async fn register(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body: Credentials = req.json().await?;
    validators::credentials(&body.email, &body.password)?;
    let name = body.name.unwrap_or_else(|| body.email.clone());
    let id = auth::register(&state.db, &body.email, &name, &body.password).await?;
    Ok(json_response!({ "id": id, "email": body.email }))
}

/// POST /auth/login — return a JWT.
pub async fn login(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body: Credentials = req.json().await?;
    validators::credentials(&body.email, &body.password)?;
    let id = auth::login(&state.db, &body.email, &body.password).await?;
    let token = auth::mint(&state.jwt, &id)?;
    Ok(json_response!({ "token": token, "id": id }))
}

/// GET /auth/me — profile behind the bearer token.
pub async fn me(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let claims = auth::authenticate(&mut req, &state.jwt).await?;
    let (id, email, name) = auth::profile(&state.db, &claims.sub).await?;
    Ok(json_response!({ "id": id, "email": email, "name": name }))
}
