use serde::{Deserialize, Serialize};
use toxi_macros::Model;

/// An author. Validation runs through `validate()` before writes.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct Author {
    pub id: i64,
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}

/// Payload for creating an author.
#[derive(Debug, Deserialize)]
pub struct CreateAuthor {
    pub name: String,
    pub email: String,
}

impl<'r> toxi::db::sqlx::FromRow<'r, toxi::db::sqlx::any::AnyRow> for Author {
    fn from_row(row: &'r toxi::db::sqlx::any::AnyRow) -> toxi::db::sqlx::Result<Self> {
        use toxi::db::sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
        })
    }
}
