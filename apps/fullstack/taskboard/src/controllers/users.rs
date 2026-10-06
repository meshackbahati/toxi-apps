//! Sample user listing with pagination.

use toxi::prelude::*;
use toxi::json_response;

#[derive(serde::Deserialize, Default)]
pub struct Pagination {
    page: Option<u64>,
    limit: Option<u64>,
}

#[derive(serde::Serialize)]
pub struct UserSummary {
    id: u64,
    name: String,
    active: bool,
}

/// GET /users — paginated sample users.
pub async fn get_users(Query(params): Query<Pagination>) -> Result<Response> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    let offset = (page - 1) * limit;
    let users: Vec<UserSummary> = ((offset + 1)..=(offset + limit))
        .map(|i| UserSummary {
            id: i,
            name: format!("User {i}"),
            active: true,
        })
        .collect();
    Ok(json_response!({
        "users": users,
        "pagination": { "page": page, "limit": limit, "offset": offset },
    }))
}
