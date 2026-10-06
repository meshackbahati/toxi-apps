//! App settings from `toxi.toml`, `.env`, and OS env, in that order.

use toxi::config::Config;
use toxi_core::{Error, Result};

/// Resolved settings for the taskboard.
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub db_url: String,
    pub jwt_secret: String,
}

/// Load settings. Every value comes through `Config`, including secrets.
pub fn load() -> Result<Settings> {
    let config =
        Config::load().map_err(|e| Error::InternalServerError(format!("config: {e}")))?;
    Ok(Settings {
        host: config.server.host.clone(),
        port: config.server.port,
        db_url: config.database.url.clone(),
        jwt_secret: config
            .get::<String>("TASKBOARD_JWT")
            .unwrap_or_else(|| "taskboard-dev-secret".to_string()),
    })
}
