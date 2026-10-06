//! Attachment files under `uploads/`, capped at 5 MB.

use toxi_core::{Error, Result};

/// Upload directory, relative to the app directory.
pub fn dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").to_path_buf()
}

/// Reject traversal and empty names.
pub fn safe_name(name: &str) -> Result<()> {
    let base = name.rsplit('/').next().unwrap_or(name);
    if base.is_empty() || base.contains("..") {
        return Err(Error::BadRequest("bad filename".to_string()));
    }
    Ok(())
}

/// Keep the original name readable while unique.
pub fn stored_name(raw: &str) -> String {
    let clean: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(64)
        .collect();
    format!("{}-{clean}", uuid::Uuid::new_v4())
}

/// Write bytes, returning the stored name.
pub async fn save(bytes: &[u8], raw: &str) -> Result<String> {
    if bytes.len() > 5 * 1024 * 1024 {
        return Err(Error::BadRequest("file over 5 MB".to_string()));
    }
    let stored = stored_name(raw);
    safe_name(&stored)?;
    tokio::fs::create_dir_all(dir())
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    tokio::fs::write(dir().join(&stored), bytes)
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(stored)
}

/// Read stored bytes.
pub async fn read(name: &str) -> Result<Vec<u8>> {
    if name.contains('/') || name.contains("..") || name.is_empty() {
        return Err(Error::BadRequest("bad filename".to_string()));
    }
    tokio::fs::read(dir().join(name))
        .await
        .map_err(|_| Error::NotFound("file not found".to_string()))
}
