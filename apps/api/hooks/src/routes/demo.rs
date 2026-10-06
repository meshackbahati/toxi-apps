//! Hook pipeline demo: audit counting plus maintenance short-circuit.
//!
//! Routes: GET /demo, POST /maintenance, GET /plugins.

use toxi::json_response;
use toxi::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::AppState;

/// GET /demo — run PreRequest hooks, report counts.
pub async fn demo(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let path = req.uri().path().to_string();
    let outcome = state
        .manager
        .execute_hook(toxi_plugin::PluginHook::PreRequest {
            path: path.clone(),
            method: "GET".to_string(),
        })
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    match outcome {
        toxi_plugin::HookResult::Stop => Ok(json_response!({ "served": false, "reason": "maintenance" })),
        _ => {
            let _ = state
                .manager
                .execute_hook(toxi_plugin::PluginHook::PostResponse {
                    path,
                    method: "GET".to_string(),
                    status: 200,
                })
                .await;
            Ok(json_response!({
                "served": true,
                "audited": state.audited.load(Ordering::Relaxed),
            }))
        }
    }
}

/// POST /maintenance — flip the short-circuit flag.
pub async fn maintenance(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let before = state.maintenance.load(Ordering::Relaxed);
    state.maintenance.store(!before, Ordering::Relaxed);
    Ok(json_response!({ "maintenance": !before }))
}

/// GET /plugins — registered plugin info.
pub async fn list(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let plugins = state.manager.list_plugins();
    Ok(json_response!({ "plugins": plugins }))
}
