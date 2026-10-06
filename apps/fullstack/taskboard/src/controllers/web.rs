//! Rendered pages: home, users, task board, API docs.

use toxi::db::{sqlx, Database};
use toxi::prelude::*;
use toxi_template::Context;
use std::sync::Arc;

use crate::models::User;
use crate::AppState;

/// GET / — home page.
pub async fn home(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let mut ctx = Context::new();
    ctx.set("title", "Taskboard");
    ctx.set("welcome_message", "Tasks, auth, realtime, uploads");
    let html = state
        .templates
        .render("home.html", &ctx)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(Response::html(html))
}

/// GET /users — rendered user directory from the database.
pub async fn users_page(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    use sqlx::Row;
    let rows = state
        .db
        .fetch_all(sqlx::query("SELECT id, email, name FROM users ORDER BY email"))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let mut users = Vec::with_capacity(rows.len());
    for row in &rows {
        users.push(User {
            id: row.try_get("id").unwrap_or_default(),
            email: row.try_get("email").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            created_at: None,
        });
    }
    let mut ctx = Context::new();
    ctx.set("users", users);
    ctx.set("user_count", rows.len() as i64);
    let html = state
        .templates
        .render("users/list.html", &ctx)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(Response::html(html))
}

/// GET /board — public task board with author names.
pub async fn board(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    use sqlx::Row;
    let rows = state
        .db
        .fetch_all(sqlx::query(
            "SELECT t.id, t.title, t.done, u.name AS author
             FROM tasks t JOIN users u ON u.id = t.user_id
             ORDER BY t.created_at DESC LIMIT 100",
        ))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let mut tasks = Vec::with_capacity(rows.len());
    for row in &rows {
        let done: i64 = row.try_get("done").unwrap_or(0);
        tasks.push(serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "title": row.try_get::<String, _>("title").unwrap_or_default(),
            "author": row.try_get::<String, _>("author").unwrap_or_default(),
            "done": done != 0,
        }));
    }
    let mut ctx = Context::new();
    ctx.set("title", "Task board");
    ctx.set("tasks", tasks);
    let html = state
        .templates
        .render("tasks/list.html", &ctx)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(Response::html(html))
}

/// GET /api-docs — human API documentation page.
pub async fn api_docs_page(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let ctx = Context::from_json(serde_json::json!({ "spec_url": "/openapi.json" }));
    let html = state
        .templates
        .render("api_docs.html", &ctx)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(Response::html(html))
}
