//! Taskboard: tasks with auth, uploads, events, rendered pages.
//!
//! Boot sequence: Config ──> Router ──> Middleware ──> Server.
//! See GUIDE.md.

use toxi::auth::JwtManager;
use toxi::db::{sqlx, Database, DbPool};
use toxi::prelude::*;
use toxi::realtime::PubSub;
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
    pub templates: Arc<toxi_template::TemplateContext>,
    pub db: DbPool,
    pub jwt: Arc<JwtManager>,
    pub bus: Arc<PubSub>,
}

impl AppState {
    async fn load(settings: &config::Settings) -> Result<Self> {
        // Run from the app directory: templates, migrations, uploads,
        // and the database resolve relative to it.
        let templates = toxi_template::TemplateContext::new("templates");
        let db = DbPool::connect(&settings.db_url)
            .await
            .map_err(|e| Error::InternalServerError(format!("db connect: {e}")))?;
        // Migrations apply once each, tracked in the database, so
        // restarts never replay DDL.
        db.execute("CREATE TABLE IF NOT EXISTS _migrations (name TEXT PRIMARY KEY)")
            .await
            .map_err(|e| Error::InternalServerError(format!("migrate: {e}")))?;
        for file in [
            "migrations/001_initial_schema.sql",
            "migrations/002_tasks.sql",
            "migrations/003_password_hash.sql",
        ] {
            let applied = db
                .fetch_one(sqlx::query("SELECT name FROM _migrations WHERE name = $1").bind(file))
                .await
                .map_err(|e| Error::InternalServerError(format!("migrate: {e}")))?;
            if applied.is_some() {
                continue;
            }
            let sql = std::fs::read_to_string(file)
                .map_err(|e| Error::InternalServerError(format!("read {file}: {e}")))?;
            for statement in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
                db.execute(statement)
                    .await
                    .map_err(|e| Error::InternalServerError(format!("migrate: {e}")))?;
            }
            db.execute_query(sqlx::query("INSERT INTO _migrations (name) VALUES ($1)").bind(file))
                .await
                .map_err(|e| Error::InternalServerError(format!("migrate: {e}")))?;
        }
        Ok(Self {
            templates: Arc::new(templates),
            db,
            jwt: Arc::new(JwtManager::new(settings.jwt_secret.clone())),
            bus: Arc::new(PubSub::new()),
        })
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
        "Taskboard on http://{}:{}",
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
        let dir = std::env::temp_dir().join(format!("taskboard-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_current_dir(&dir).unwrap();
        std::fs::create_dir_all("templates").unwrap();
        std::fs::write("templates/home.html", "hi").unwrap();
        std::fs::create_dir_all("migrations").unwrap();
        for f in [
            "migrations/001_initial_schema.sql",
            "migrations/002_tasks.sql",
            "migrations/003_password_hash.sql",
        ] {
            std::fs::copy(format!("{}/{f}", env!("CARGO_MANIFEST_DIR")), f).unwrap();
        }
        let settings = config::Settings {
            host: "127.0.0.1".to_string(),
            port: 3000,
            db_url: "sqlite:taskboard-test.db".to_string(),
            jwt_secret: "test-secret".to_string(),
        };
        Arc::new(AppState::load(&settings).await.unwrap())
    }

    #[tokio::test]
    async fn full_flow() {
        let state = test_state().await;

        let res = controllers::auth::register(
            json_req(
                "POST",
                "/auth/register",
                br#"{"email":"t@t.c","password":"password1","name":"T"}"#,
                &state,
            ),
        )
        .await
        .unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);

        let req = json_req(
            "POST",
            "/auth/login",
            br#"{"email":"t@t.c","password":"password1"}"#,
            &state,
        );
        let res = controllers::auth::login(req).await.unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);

        let res = controllers::tasks::create(
            json_req("POST", "/tasks", br#"{"title":"x"}"#, &state),
        )
        .await;
        // No token yet: must be unauthorized.
        assert!(res.is_err());

        let res = controllers::docs::openapi_spec(json_req("GET", "/openapi.json", b"", &state))
            .await
            .unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);
    }
}
