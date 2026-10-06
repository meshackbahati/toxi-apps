//! Task endpoints. Validation first, then the task service.

use toxi::json_response;
use toxi::prelude::*;
use toxi_core::extract::PathParams;
use toxi_core::request::RequestExt;
use std::sync::Arc;

use crate::models::{CreateTask, UpdateTask};
use crate::services::{auth, tasks};
use crate::validators;
use crate::AppState;

fn id_of(req: &Request) -> String {
    req.extensions()
        .get::<PathParams>()
        .and_then(|p| p.0.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// GET /tasks — my tasks, newest first.
pub async fn list(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let claims = auth::authenticate(&mut req, &state.jwt).await?;
    Ok(json_response!({ "tasks": tasks::list(&state.db, &claims.sub).await? }))
}

/// POST /tasks — create one and announce it on the bus.
pub async fn create(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let claims = auth::authenticate(&mut req, &state.jwt).await?;
    let body: CreateTask = req.json().await?;
    validators::task(&body.title)?;
    let task = tasks::create(&state.db, &claims.sub, &body).await?;
    let _ = state
        .bus
        .publish_message("tasks", serde_json::json!({ "event": "created", "id": task.id }))
        .await;
    Ok(json_response!({ "task": task }))
}

/// GET /tasks/:id — one of my tasks.
pub async fn get(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let claims = auth::authenticate(&mut req, &state.jwt).await?;
    Ok(json_response!({ "task": tasks::one(&state.db, &id_of(&req), &claims.sub).await? }))
}

/// PUT /tasks/:id — partial update.
pub async fn update(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let claims = auth::authenticate(&mut req, &state.jwt).await?;
    let body: UpdateTask = req.json().await?;
    if let Some(title) = &body.title {
        validators::task(title)?;
    }
    tasks::update(&state.db, &id_of(&req), &claims.sub, &body).await?;
    Ok(json_response!({ "id": id_of(&req) }))
}

/// DELETE /tasks/:id — remove one task.
pub async fn delete(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let claims = auth::authenticate(&mut req, &state.jwt).await?;
    let id = id_of(&req);
    tasks::delete(&state.db, &id, &claims.sub).await?;
    Ok(json_response!({ "deleted": id }))
}
