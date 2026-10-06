//! Input validation. Controllers reject bad input here, before services run.

use toxi_core::{Error, Result};

/// Email shape plus password length.
pub fn credentials(email: &str, password: &str) -> Result<()> {
    if !email.contains('@') || email.len() > 320 {
        return Err(Error::BadRequest("invalid email".to_string()));
    }
    if password.len() < 8 {
        return Err(Error::BadRequest("password needs 8+ characters".to_string()));
    }
    Ok(())
}

/// Task title bounds.
pub fn task(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(Error::BadRequest("title is empty".to_string()));
    }
    if title.len() > 200 {
        return Err(Error::BadRequest("title over 200 chars".to_string()));
    }
    Ok(())
}
