//! Task persistence. Controllers stay thin; SQL lives here.

use toxi::db::{sqlx, Database, DbPool};
use toxi_core::{Error, Result};

use crate::models::{CreateTask, Task, UpdateTask};

fn row_task(row: &sqlx::any::AnyRow) -> Result<Task> {
    use sqlx::Row;
    Ok(Task {
        id: row.try_get("id").map_err(|e| Error::InternalServerError(e.to_string()))?,
        user_id: row.try_get("user_id").map_err(|e| Error::InternalServerError(e.to_string()))?,
        title: row.try_get("title").map_err(|e| Error::InternalServerError(e.to_string()))?,
        done: row.try_get::<i64, _>("done").map_err(|e| Error::InternalServerError(e.to_string()))? != 0,
        created_at: row.try_get("created_at").ok(),
    })
}

/// Tasks for one owner, newest first.
pub async fn list(db: &DbPool, user_id: &str) -> Result<Vec<Task>> {
    let rows = db
        .fetch_all(sqlx::query("SELECT id, user_id, title, done, created_at FROM tasks WHERE user_id = $1 ORDER BY created_at DESC").bind(user_id))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    rows.iter().map(row_task).collect()
}

/// Insert a task, returning it.
pub async fn create(db: &DbPool, user_id: &str, body: &CreateTask) -> Result<Task> {
    let task = Task::new(user_id.to_string(), body.title.trim().to_string());
    db.execute_query(
        sqlx::query("INSERT INTO tasks (id, user_id, title, done) VALUES ($1, $2, $3, 0)")
            .bind(&task.id)
            .bind(&task.user_id)
            .bind(&task.title),
    )
    .await
    .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(task)
}

/// One task by id and owner.
pub async fn one(db: &DbPool, id: &str, user_id: &str) -> Result<Task> {
    let row = db
        .fetch_one(sqlx::query("SELECT id, user_id, title, done, created_at FROM tasks WHERE id = $1 AND user_id = $2").bind(id).bind(user_id))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let row = row.ok_or_else(|| Error::NotFound("task not found".to_string()))?;
    row_task(&row)
}

/// Apply a partial update.
pub async fn update(db: &DbPool, id: &str, user_id: &str, body: &UpdateTask) -> Result<()> {
    if let Some(title) = &body.title {
        db.execute_query(sqlx::query("UPDATE tasks SET title = $1 WHERE id = $2 AND user_id = $3").bind(title).bind(id).bind(user_id))
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;
    }
    if let Some(done) = body.done {
        db.execute_query(sqlx::query("UPDATE tasks SET done = $1 WHERE id = $2 AND user_id = $3").bind(if done { 1i64 } else { 0i64 }).bind(id).bind(user_id))
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;
    }
    Ok(())
}

/// Delete one task.
pub async fn delete(db: &DbPool, id: &str, user_id: &str) -> Result<()> {
    db.execute_query(sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2").bind(id).bind(user_id))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(())
}
