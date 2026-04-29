use time::OffsetDateTime;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::entity::history::{AuthorHistory, BookHistory},
};

#[derive(Debug, Clone)]
pub struct BookHistoryDto {
    pub history_id: i64,
    pub change_set_id: String,
    pub operation: String,
    pub book_id: String,
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub book_created_at: OffsetDateTime,
    pub book_updated_at: OffsetDateTime,
    pub changed_at: OffsetDateTime,
}

impl From<BookHistory> for BookHistoryDto {
    fn from(h: BookHistory) -> Self {
        Self {
            history_id: h.history_id,
            change_set_id: h.change_set_id.to_string(),
            operation: h.operation.as_str().to_string(),
            book_id: h.book_id.to_string(),
            title: h.title.into_string(),
            author_ids: h.author_ids.into_iter().map(|a| a.to_string()).collect(),
            isbn: h.isbn.into_string(),
            read: h.read.to_bool(),
            owned: h.owned.to_bool(),
            priority: h.priority.to_i32(),
            format: h.format,
            store: h.store,
            book_created_at: h.book_created_at,
            book_updated_at: h.book_updated_at,
            changed_at: h.changed_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthorHistoryDto {
    pub history_id: i64,
    pub change_set_id: String,
    pub operation: String,
    pub author_id: String,
    pub name: String,
    pub yomi: String,
    pub author_created_at: OffsetDateTime,
    pub author_updated_at: OffsetDateTime,
    pub changed_at: OffsetDateTime,
}

impl From<AuthorHistory> for AuthorHistoryDto {
    fn from(h: AuthorHistory) -> Self {
        Self {
            history_id: h.history_id,
            change_set_id: h.change_set_id.to_string(),
            operation: h.operation.as_str().to_string(),
            author_id: h.author_id.to_string(),
            name: h.name,
            yomi: h.yomi,
            author_created_at: h.author_created_at,
            author_updated_at: h.author_updated_at,
            changed_at: h.changed_at,
        }
    }
}
