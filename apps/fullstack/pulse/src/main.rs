//! Pulse: request metrics dashboard.
//!
//! A recorder middleware feeds the global registry; `/` renders it,
//! `/metrics` serves Prometheus text. See GUIDE.md.

use toxi::prelude::*;
use toxi_utils::metrics::MetricsRegistry;
use std::sync::Arc;

mod middleware;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub templates: Arc<toxi_template::TemplateContext>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(AppState {
        templates: Arc::new(toxi_template::TemplateContext::new("templates")),
    });
    let mut router = Router::new();

    router.get("/", routes::dashboard::dashboard);
    router.get("/metrics", routes::dashboard::prometheus);
    router.get("/api/status", routes::dashboard::api_status);
    router.get("/health", routes::dashboard::health_check);
    // Static assets last: specific routes match first, everything else
    // falls through to the template engine file server.
    router.get("/*", toxi_template::serve_static);

    let mut router = router;
    router.with_state(state);
    // Recorder wraps the router so every request is counted.
    let recorded = middleware::Recorder::new(router);
    println!("Pulse on http://127.0.0.1:3007");
    Server::new(recorded).listen("127.0.0.1:3007".parse().unwrap()).await
}

#[cfg(test)]
mod boot_tests {
    use super::*;

    #[tokio::test]
    async fn metrics_record() {
        let registry = MetricsRegistry::new();
        registry.record_request("/x", 5, true);
        let snap = registry.get_snapshot();
        assert_eq!(snap["/x"].0, 1);
        let text = toxi_utils::metrics::render_prometheus();
        assert!(text.contains("toxi_requests_total"));
    }
}
