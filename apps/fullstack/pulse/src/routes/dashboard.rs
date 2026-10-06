//! Dashboard plus Prometheus text.
//!
//! Routes: GET /, GET /metrics, GET /api/status, GET /health.

use toxi::json_response;
use toxi::prelude::*;
use toxi_template::Context;
use std::sync::Arc;

use crate::AppState;

/// GET / — dashboard table of per-route counters.
pub async fn dashboard(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let snapshot = toxi_utils::metrics::GLOBAL_METRICS.get_snapshot();
    let mut rows: Vec<serde_json::Value> = snapshot
        .iter()
        .map(|(path, (count, success, errors, ms))| {
            serde_json::json!({
                "path": path,
                "requests": count,
                "ok": success,
                "errors": errors,
                "ms": ms,
            })
        })
        .collect();
    rows.sort_by(|a, b| b["requests"].as_u64().cmp(&a["requests"].as_u64()));
    let mut ctx = Context::new();
    ctx.set("title", "Pulse");
    ctx.set("rows", rows);
    let html = state
        .templates
        .render("dashboard.html", &ctx)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(Response::html(html))
}

/// GET /metrics — Prometheus text for scrapers.
pub async fn prometheus(_req: Request) -> Result<Response> {
    let body = toxi_utils::metrics::render_prometheus();
    let res = http::Response::builder()
        .header(http::header::CONTENT_TYPE, "text/plain; version=0.0.4")
        .body(http_body_util::Full::new(bytes::Bytes::from(body)).map_err(|e| match e {}).boxed())
        .map_err(|e| Error::InternalServerError(format!("build: {e}")))?;
    Ok(toxi_core::ToxiResponse::new(res))
}

/// GET /api/status — liveness plus build identity.
pub async fn api_status(_req: Request) -> Result<Response> {
    Ok(json_response!({ "status": "online", "app": "pulse" }))
}

/// GET /health — minimal warmth probe.
pub async fn health_check(_req: Request) -> Result<Response> {
    Ok(Response::text("OK"))
}
