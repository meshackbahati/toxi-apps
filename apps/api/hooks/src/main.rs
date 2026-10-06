//! Hooks: in-process plugin pipeline.
//!
//! Audit counts requests, maintenance short-circuits them. See GUIDE.md.

use toxi::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};

mod plugins;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub manager: Arc<toxi_plugin::PluginManager>,
    pub audited: Arc<AtomicU64>,
    pub maintenance: Arc<AtomicBool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let audited = Arc::new(AtomicU64::new(0));
    let maintenance = Arc::new(AtomicBool::new(false));
    let mut manager = toxi_plugin::PluginManager::new(toxi_plugin::PluginConfig::default());
    manager
        .register_plugin(Arc::new(plugins::audit::AuditPlugin::new(audited.clone())))
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    manager
        .register_plugin(Arc::new(plugins::maintenance::MaintenancePlugin::new(
            maintenance.clone(),
        )))
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    let state = Arc::new(AppState {
        manager: Arc::new(manager),
        audited,
        maintenance,
    });
    let mut router = Router::new();

    router.get("/health", routes::status::health_check);
    router.get("/api/status", routes::status::api_status);
    router.get("/demo", routes::demo::demo);
    router.post("/maintenance", routes::demo::maintenance);
    router.get("/plugins", routes::demo::list);

    let mut router = router;
    router.with_state(state);
    println!("Hooks on http://127.0.0.1:3008");
    Server::new(router).listen("127.0.0.1:3008".parse().unwrap()).await
}

#[cfg(test)]
mod boot_tests {
    use super::*;

    fn state() -> Arc<AppState> {
        let audited = Arc::new(AtomicU64::new(0));
        let maintenance = Arc::new(AtomicBool::new(false));
        let mut manager = toxi_plugin::PluginManager::new(toxi_plugin::PluginConfig::default());
        manager
            .register_plugin(Arc::new(plugins::audit::AuditPlugin::new(audited.clone())))
            .unwrap();
        manager
            .register_plugin(Arc::new(plugins::maintenance::MaintenancePlugin::new(
                maintenance.clone(),
            )))
            .unwrap();
        Arc::new(AppState {
            manager: Arc::new(manager),
            audited,
            maintenance,
        })
    }

    fn req(state: &Arc<AppState>) -> Request {
        let mut req = http::Request::builder()
            .method("GET")
            .uri("/demo")
            .body(
                http_body_util::Full::new(bytes::Bytes::new())
                    .map_err(|e| match e {})
                    .boxed(),
            )
            .unwrap();
        req.extensions_mut().insert(state.clone());
        req
    }

    #[tokio::test]
    async fn full_flow() {
        use http_body_util::BodyExt;
        let state = state();

        let res = routes::demo::demo(req(&state)).await.unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);

        let res = routes::demo::maintenance(req(&state)).await.unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);

        // Maintenance active: demo short-circuits.
        let res = routes::demo::demo(req(&state)).await.unwrap();
        assert_eq!(res.status(), http::StatusCode::OK);
        assert_eq!(state.audited.load(std::sync::atomic::Ordering::Relaxed), 2);
    }
}
