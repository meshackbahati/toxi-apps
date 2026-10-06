use serde::{Deserialize, Serialize};
use toxi_macros::Model;

/// A book belonging to an author.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct Book {
    pub id: i64,
    pub author_id: i64,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
}

/// Payload for creating a book.
#[derive(Debug, Deserialize)]
pub struct CreateBook {
    pub author_id: i64,
    pub title: String,
}

impl<'r> toxi::db::sqlx::FromRow<'r, toxi::db::sqlx::any::AnyRow> for Book {
    fn from_row(row: &'r toxi::db::sqlx::any::AnyRow) -> toxi::db::sqlx::Result<Self> {
        use toxi::db::sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            author_id: row.try_get("author_id")?,
            title: row.try_get("title")?,
        })
    }
}
