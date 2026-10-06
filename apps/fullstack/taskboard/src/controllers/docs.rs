//! Machine-readable API surface: OpenAPI spec plus favicon.

use toxi::prelude::*;

/// Serve the OpenAPI spec for the taskboard API.
pub async fn openapi_spec(_req: Request) -> Result<Response> {
    use toxi_openapi::{OpenApiBuilder, Operation, PathItem, Response as OpenApiResponse};

    let mut builder =
        OpenApiBuilder::new("Taskboard API", "0.1.0").description("Tasks with auth and uploads");

    let get = |summary: &str, description: &str| {
        let mut map = std::collections::HashMap::new();
        map.insert(
            "200".to_string(),
            OpenApiResponse {
                description: description.to_string(),
                content: None,
            },
        );
        Operation {
            summary: Some(summary.to_string()),
            description: Some(description.to_string()),
            responses: map,
            ..Default::default()
        }
    };

    builder = builder.path(
        "/tasks",
        PathItem {
            get: Some(get("List tasks", "Tasks for the bearer-token owner")),
            post: Some(get("Create task", "Create a task for the owner")),
            ..Default::default()
        },
    );
    builder = builder.path(
        "/auth/login",
        PathItem {
            get: None,
            post: Some(get("Login", "Verify credentials, return a JWT")),
            ..Default::default()
        },
    );

    let spec = builder.build();
    Ok(toxi_core::ToxiResponse::json(spec))
}

/// Favicon handler.
pub async fn favicon(_req: Request) -> Result<Response> {
    use http_body_util::BodyExt;
    let content =
        std::fs::read("public/images/toxi.svg").map_err(|_| Error::NotFound("icon".to_string()))?;
    let res = http::Response::builder()
        .header(http::header::CONTENT_TYPE, "image/svg+xml")
        .body(http_body_util::Full::new(bytes::Bytes::from(content)).map_err(|e| match e {}).boxed())
        .map_err(|e| Error::InternalServerError(format!("build: {e}")))?;
    Ok(toxi_core::ToxiResponse::new(res))
}
