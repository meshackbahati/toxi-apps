//! Author and book endpoints, all through the derived Model API.
//!
//! Routes: GET/POST /authors, GET/POST /books, GET /authors/:id/books.

use toxi::db::Model;
use toxi::json_response;
use toxi_core::extract::PathParams;
use toxi_core::request::RequestExt;
use toxi_core::{Error, FromRequest, Request, Response, Result, State};
use std::sync::Arc;

use crate::models::{Author, Book, CreateAuthor, CreateBook};
use crate::AppState;

fn id_of(req: &Request) -> String {
    req.extensions()
        .get::<PathParams>()
        .and_then(|p| p.0.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// GET /authors — all authors via the derived query.
pub async fn authors(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let list = Author::all(&state.db)
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(json_response!({ "authors": list }))
}

/// POST /authors — validate, then insert.
pub async fn create_author(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body: CreateAuthor = req.json().await?;
    let mut author = Author {
        id: uuid::Uuid::new_v4().to_string(),
        name: body.name,
        email: body.email,
    };
    author
        .validate(&state.db)
        .await
        .map_err(Error::BadRequest)?;
    author
        .create(&state.db)
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(json_response!({ "author": author }))
}

/// GET /books — all books via the derived query.
pub async fn books(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let list = Book::all(&state.db)
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(json_response!({ "books": list }))
}

/// POST /books — validate, then insert.
pub async fn create_book(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let body: CreateBook = req.json().await?;
    let mut book = Book {
        id: uuid::Uuid::new_v4().to_string(),
        author_id: body.author_id,
        title: body.title,
    };
    book.validate(&state.db)
        .await
        .map_err(Error::BadRequest)?;
    book.create(&state.db)
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(json_response!({ "book": book }))
}

/// GET /authors/:id/books — books for one author, filtered query.
pub async fn author_books(mut req: Request) -> Result<Response> {
    let State(state): State<Arc<AppState>> = State::from_request(&mut req).await?;
    let id = id_of(&req);
    let list = Book::query()
        .filter_eq("author_id", &id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| Error::InternalServerError(e.to_string()))?;
    Ok(json_response!({ "books": list }))
}
