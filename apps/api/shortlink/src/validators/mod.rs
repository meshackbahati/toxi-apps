//! Input validation. Controllers reject bad input here, before services run.

use toxi_core::{Error, Result};

/// Path codes are short, alphanumeric, dash/underscore only.
pub fn code(code: &str) -> Result<()> {
    if code.is_empty()
        || code.len() > 32
        || !code
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(Error::NotFound("unknown code".to_string()));
    }
    Ok(())
}

/// Targets must be http(s) and bounded.
pub fn url(url: &str) -> Result<()> {
    if !(url.starts_with("http://") || url.starts_with("https://")) || url.len() > 2048 {
        return Err(Error::BadRequest("url must be http(s), max 2048 chars".to_string()));
    }
    Ok(())
}

/// Custom codes validate like path codes, but fail as bad requests.
pub fn custom(value: &str) -> Result<()> {
    if !value.is_empty() {
        code(value).map_err(|_| Error::BadRequest("bad custom code".to_string()))?;
    }
    Ok(())
}
