//! Link persistence plus the cache-first read path.

use toxi::cache::{Cache, MemoryCache};
use toxi::db::{sqlx, Database, DbPool};
use toxi_core::{Error, Result};
use std::sync::Arc;
use std::time::Duration;

use crate::models::Link;

const CACHE_TTL: Duration = Duration::from_secs(3600);

/// Insert a code, retrying random codes on collision.
pub async fn insert(db: &DbPool, cache: &MemoryCache, url: &str, wanted: &str) -> Result<String> {
    for _ in 0..5 {
        let code = if wanted.is_empty() {
            crate::models::random_code()
        } else {
            wanted.to_string()
        };
        let done = db
            .execute_query(sqlx::query("INSERT INTO links (code, url) VALUES ($1, $2)").bind(&code).bind(url))
            .await;
        match done {
            Ok(_) => {
                let _ = cache.set(&code, &url.to_string(), Some(CACHE_TTL)).await;
                return Ok(code);
            }
            Err(_) => {
                if !wanted.is_empty() {
                    return Err(Error::Conflict("code taken".to_string()));
                }
            }
        }
    }
    Err(Error::InternalServerError("code collision, retry".to_string()))
}

/// Resolve a code, cache first, database on miss.
pub async fn resolve(db: &DbPool, cache: &MemoryCache, code: &str) -> Result<Option<String>> {
    if let Ok(Some(url)) = cache.get::<String>(code).await {
        return Ok(Some(url));
    }
    let row = db
        .fetch_one(sqlx::query("SELECT url FROM links WHERE code = $1").bind(code))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    match row {
        None => Ok(None),
        Some(row) => {
            use sqlx::Row;
            let url: String = row.try_get("url").map_err(|e| Error::InternalServerError(e.to_string()))?;
            let _ = cache.set(code, &url, Some(CACHE_TTL)).await;
            Ok(Some(url))
        }
    }
}

/// Bump the hit counter. Fire and forget by the caller.
pub async fn bump(db: &DbPool, code: &str) {
    let _ = db
        .execute_query(sqlx::query("UPDATE links SET hits = hits + 1 WHERE code = $1").bind(code))
        .await;
}

/// Newest links first, capped at 100.
pub async fn list(db: &DbPool) -> Result<Vec<Link>> {
    use sqlx::Row;
    let rows = db
        .fetch_all(sqlx::query("SELECT code, url, hits FROM links ORDER BY created_at DESC LIMIT 100"))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let mut links = Vec::with_capacity(rows.len());
    for row in &rows {
        links.push(Link {
            code: row.try_get("code").unwrap_or_default(),
            url: row.try_get("url").unwrap_or_default(),
            hits: row.try_get("hits").unwrap_or(0),
        });
    }
    Ok(links)
}

/// Totals for the stats endpoint.
pub async fn counts(db: &DbPool) -> Result<(i64, i64)> {
    use sqlx::Row;
    let row = db
        .fetch_one(sqlx::query("SELECT COUNT(*) AS n, COALESCE(SUM(hits), 0) AS h FROM links"))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok((
        row.as_ref().and_then(|r| r.try_get("n").ok()).unwrap_or(0),
        row.as_ref().and_then(|r| r.try_get("h").ok()).unwrap_or(0),
    ))
}
