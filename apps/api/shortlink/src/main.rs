use toxi::cache::MemoryCache;
use toxi::db::DbPool;
use toxi::middleware::{RateLimitConfig, RateLimiter};
use toxi::prelude::*;
use std::sync::Arc;

mod config;
mod controllers;
mod middleware;
mod models;
mod routes;
mod services;
mod validators;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub cache: Arc<MemoryCache>,
    pub limiter: Arc<RateLimiter>,
}

impl AppState {
    async fn load(settings: &config::Settings) -> Result<Self> {
        let db_url = settings.db_url.clone();
        let db = DbPool::connect(&db_url)
            .await
            .map_err(|e| Error::InternalServerError(format!("db connect: {e}")))?;
        let sql = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations/001_links.sql"))
            .map_err(|e| Error::InternalServerError(format!("read migration: {e}")))?;
        for statement in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            db.execute(statement)
                .await
                .map_err(|e| Error::InternalServerError(format!("migrate: {e}")))?;
        }
        Ok(Self {
            db,
            cache: Arc::new(MemoryCache::new()),
            limiter: Arc::new(RateLimiter::new(RateLimitConfig {
                requests_per_minute: 60,
                requests_per_hour: Some(1000),
            })),
        })
    }
}

#[cfg(test)]
mod boot_tests {
    use super::*;
    use http_body_util::BodyExt;

    fn json_req(method: &str, uri: &str, body: &[u8], state: &Arc<AppState>) -> Request {
        let mut req = http::Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(
                http_body_util::Full::new(bytes::Bytes::from(body.to_vec()))
                    .map_err(|e| match e {})
                    .boxed(),
            )
            .unwrap();
        req.extensions_mut().insert(state.clone());
        req
    }

    async fn test_state() -> Arc<AppState> {
        let dir = std::env::temp_dir().join(format!("shortlink-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("migrations")).unwrap();
        std::env::set_current_dir(&dir).unwrap();
        std::fs::copy(
            format!("{}/migrations/001_links.sql", env!("CARGO_MANIFEST_DIR")),
            "migrations/001_links.sql",
        )
        .unwrap();
        std::env::set_var("SHORTLINK_DB", "sqlite:shortlink-test.db");
        Arc::new(AppState::load(&config::Settings { host: "127.0.0.1".to_string(), port: 3001, db_url: "sqlite:shortlink-test.db".to_string() }).await.unwrap())
    }

    #[tokio::test]
    async fn full_flow() {
        let state = test_state().await;

        let res = controllers::links::create(
            json_req("POST", "/links", br#"{"url":"https://example.com"}"#, &state),
        )
        .await
        .unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);

        let res = controllers::links::create(
            json_req("POST", "/links", br#"{"url":"not-a-url"}"#, &state),
        )
        .await;
        assert!(res.is_err());

        let res = controllers::links::stats(json_req("GET", "/stats", b"", &state))
            .await
            .unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // ── 1. Config ──────────────────────────────────────────────────
    let settings = config::load()?;

    // Shared state travels in request extensions for State.
    let state = Arc::new(AppState::load(&settings).await?);

    // ── 2. Router ──────────────────────────────────────────────────
    let mut app = Application::new(
        toxi::config::Config::load()
            .map_err(|e| Error::InternalServerError(format!("config: {e}")))?,
    );
    routes::register(app.router_mut());
    app.router_mut().with_state(state);

    // ── 3. Middleware ── 4. Server ─────────────────────────────────
    println!(
        "Shortlink on http://{}:{}",
        app.config().server.host,
        app.config().server.port
    );
    let router = app.into_router();
    let logged = middleware::Logger::new(router);
    let addr: std::net::SocketAddr = format!("{}:{}", settings.host, settings.port)
        .parse()
        .unwrap();
    Server::new(logged).listen(addr).await
}
