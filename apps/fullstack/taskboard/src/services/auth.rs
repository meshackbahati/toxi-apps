//! Credential handling: registration, login, token checks.

use toxi::auth::{Claims, JwtManager, hash_password, verify_password};
use toxi::db::{sqlx, Database, DbPool};
use toxi_core::{Error, Request, Result};

/// Read the bearer token and verify it against app state.
pub async fn authenticate(req: &mut Request, jwt: &JwtManager) -> Result<Claims> {
    let header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = header.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        return Err(Error::Unauthorized("missing bearer token".to_string()));
    }
    jwt.verify(token)
        .map_err(|_| Error::Unauthorized("invalid token".to_string()))
}

/// Register a user, returning the new id.
pub async fn register(db: &DbPool, email: &str, name: &str, password: &str) -> Result<String> {
    let hash = hash_password(password)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let user = crate::models::User::new(email.to_string(), name.to_string());
    let id = user.id.clone();
    db.execute_query(
        sqlx::query("INSERT INTO users (id, email, name, password_hash) VALUES ($1, $2, $3, $4)")
            .bind(&id)
            .bind(&user.email)
            .bind(&user.name)
            .bind(&hash),
    )
    .await
    .map_err(|e| Error::BadRequest(format!("register failed: {e}")))?;
    Ok(id)
}

/// Verify credentials, returning the user id.
pub async fn login(db: &DbPool, email: &str, password: &str) -> Result<String> {
    use sqlx::Row;
    let row = db
        .fetch_one(sqlx::query("SELECT id, password_hash FROM users WHERE email = $1").bind(email))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let row = row.ok_or_else(|| Error::Unauthorized("unknown email".to_string()))?;
    let id: String = row.try_get("id").map_err(|e| Error::InternalServerError(e.to_string()))?;
    let hash: String = row
        .try_get("password_hash")
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    if !verify_password(password, &hash).map_err(|e| Error::InternalServerError(e.to_string()))? {
        return Err(Error::Unauthorized("wrong password".to_string()));
    }
    Ok(id)
}

/// Profile fields for a user id.
pub async fn profile(db: &DbPool, user_id: &str) -> Result<(String, String, String)> {
    use sqlx::Row;
    let row = db
        .fetch_one(sqlx::query("SELECT id, email, name FROM users WHERE id = $1").bind(user_id))
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    let row = row.ok_or_else(|| Error::Unauthorized("user gone".to_string()))?;
    Ok((
        row.try_get("id").unwrap_or_default(),
        row.try_get("email").unwrap_or_default(),
        row.try_get("name").unwrap_or_default(),
    ))
}

/// Mint a JWT for a user id.
pub fn mint(jwt: &JwtManager, user_id: &str) -> Result<String> {    jwt.generate_token(&Claims::new(user_id.to_string(), 86400))
        .map_err(|e| Error::InternalServerError(e.to_string()))
}
