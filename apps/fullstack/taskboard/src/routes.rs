//! Route table. Controllers stay in `controllers/`, wiring lives here.

use toxi::prelude::*;

use crate::controllers;

/// Register every route on the application router.
pub fn register(router: &mut Router) {
    router.get("/", controllers::web::home);
    router.get("/users", controllers::web::users_page);
    router.get("/api/users", controllers::users::get_users);
    router.get("/board", controllers::web::board);
    router.get("/api-docs", controllers::web::api_docs_page);
    router.get("/api/status", controllers::status::api_status);
    router.get("/health", controllers::status::health_check);

    router.post("/auth/register", controllers::auth::register);
    router.post("/auth/login", controllers::auth::login);
    router.get("/auth/me", controllers::auth::me);

    router.get("/tasks", controllers::tasks::list);
    router.post("/tasks", controllers::tasks::create);
    router.get("/tasks/:id", controllers::tasks::get);
    router.put("/tasks/:id", controllers::tasks::update);
    router.delete("/tasks/:id", controllers::tasks::delete);

    router.post("/uploads", controllers::uploads::upload);
    router.get("/uploads/:name", controllers::uploads::download);
    router.get("/events/next", controllers::realtime::next);
    router.get("/openapi.json", controllers::docs::openapi_spec);
    router.get("/favicon.ico", controllers::docs::favicon);
    // Static assets last: specific routes match first, everything else
    // falls through to the template engine file server.
    router.get("/*", toxi_template::serve_static);
}
