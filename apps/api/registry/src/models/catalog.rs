//! Author and book models. The derive reads these structs and
//! generates table mapping, CRUD, validation, and queries.

use serde::{Deserialize, Serialize};
use toxi_macros::Model;

/// An author. Validation runs through `validate()` before writes.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct Author {
    pub id: String,
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}

/// A book belonging to an author.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct Book {
    pub id: String,
    pub author_id: String,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
}

/// Payload for creating an author.
#[derive(Debug, Deserialize)]
pub struct CreateAuthor {
    pub name: String,
    pub email: String,
}

/// Payload for creating a book.
#[derive(Debug, Deserialize)]
pub struct CreateBook {
    pub author_id: String,
    pub title: String,
}
