//! Registry: authors and books through `#[derive(Model)]`.
//!
//! Table mapping, validation, and queries all come from the derive.
//! See GUIDE.md.

use toxi::db::DbPool;
use toxi::prelude::*;
use std::sync::Arc;

mod models;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

impl AppState {
    async fn load() -> Result<Self> {
        let db_url =
            std::env::var("REGISTRY_DB").unwrap_or_else(|_| "sqlite:registry.db".to_string());
        let db = DbPool::connect(&db_url)
            .await
            .map_err(|e| Error::InternalServerError(format!("db connect: {e}")))?;
        let sql = std::fs::read_to_string("migrations/001_registry.sql")
            .map_err(|e| Error::InternalServerError(format!("read migration: {e}")))?;
        for statement in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            db.execute(statement)
                .await
                .map_err(|e| Error::InternalServerError(format!("migrate: {e}")))?;
        }
        Ok(Self { db })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(AppState::load().await?);
    let mut router = Router::new();

    router.get("/health", routes::status::health_check);
    router.get("/api/status", routes::status::api_status);
    router.get("/authors", routes::catalog::authors);
    router.post("/authors", routes::catalog::create_author);
    router.get("/books", routes::catalog::books);
    router.post("/books", routes::catalog::create_book);
    router.get("/authors/:id/books", routes::catalog::author_books);

    let mut router = router;
    router.with_state(state);
    println!("Registry on http://127.0.0.1:3009");
    Server::new(router).listen("127.0.0.1:3009".parse().unwrap()).await
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
        let dir = std::env::temp_dir().join(format!("registry-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("migrations")).unwrap();
        std::env::set_current_dir(&dir).unwrap();
        std::fs::copy(
            format!("{}/migrations/001_registry.sql", env!("CARGO_MANIFEST_DIR")),
            "migrations/001_registry.sql",
        )
        .unwrap();
        std::env::set_var("REGISTRY_DB", "sqlite:registry-test.db");
        Arc::new(AppState::load().await.unwrap())
    }

    #[tokio::test]
    async fn full_flow() {
        let state = test_state().await;

        // Bad email fails validation before any insert.
        let res = routes::catalog::create_author(
            json_req("POST", "/authors", br#"{"name":"A","email":"nope"}"#, &state),
        )
        .await;
        assert!(res.is_err());

        let res = routes::catalog::create_author(
            json_req("POST", "/authors", br#"{"name":"A","email":"a@b.c"}"#, &state),
        )
        .await
        .unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);

        let res = routes::catalog::authors(json_req("GET", "/authors", b"", &state))
            .await
            .unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);
    }
}
