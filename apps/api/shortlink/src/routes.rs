//! Route table. Controllers stay in `controllers/`, wiring lives here.

use toxi::prelude::*;

use crate::controllers;

/// Register every route on the application router.
pub fn register(router: &mut Router) {
    router.get("/health", controllers::status::health_check);
    router.get("/api/status", controllers::status::api_status);
    router.post("/links", controllers::links::create);
    router.get("/links", controllers::links::list);
    router.get("/stats", controllers::links::stats);
    router.get("/:code", controllers::links::redirect);
}
