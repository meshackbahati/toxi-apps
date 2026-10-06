//! Service status endpoints.

use toxi::prelude::*;
use toxi::json_response;

/// GET /api/status — liveness plus build identity.
pub async fn api_status(_req: Request) -> Result<Response> {
    Ok(json_response!({ "status": "online", "app": "hooks" }))
}

/// GET /health — minimal warmth probe.
pub async fn health_check(_req: Request) -> Result<Response> {
    Ok(Response::text("OK"))
}
