use serde_json::Value;
use time::OffsetDateTime;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::entity::event::{AuthorEvent, BookEvent},
};

#[derive(Debug, Clone)]
pub struct EventSetDto {
    pub id: String,
    pub user_id: String,
    pub operation: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct BookEventDto {
    pub event_id: i64,
    pub event_set_id: String,
    pub operation: String,
    pub book_id: String,
    pub title: Option<String>,
    pub author_ids: Vec<String>,
    pub isbn: Option<String>,
    pub read: Option<bool>,
    pub owned: Option<bool>,
    pub priority: Option<i32>,
    pub format: Option<BookFormat>,
    pub store: Option<BookStore>,
    pub book_created_at: Option<OffsetDateTime>,
    pub book_updated_at: Option<OffsetDateTime>,
    pub changed_at: OffsetDateTime,
    pub extra: Option<Value>,
}

impl From<BookEvent> for BookEventDto {
    fn from(e: BookEvent) -> Self {
        Self {
            event_id: e.event_id,
            event_set_id: e.event_set_id.to_string(),
            operation: e.operation.as_str().to_string(),
            book_id: e.book_id.to_string(),
            title: e.title.map(|t| t.into_string()),
            author_ids: e.author_ids.into_iter().map(|a| a.to_string()).collect(),
            isbn: e.isbn.map(|i| i.into_string()),
            read: e.read.map(|r| r.to_bool()),
            owned: e.owned.map(|o| o.to_bool()),
            priority: e.priority.map(|p| p.to_i32()),
            format: e.format,
            store: e.store,
            book_created_at: e.book_created_at,
            book_updated_at: e.book_updated_at,
            changed_at: e.changed_at,
            extra: e.extra,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthorEventDto {
    pub event_id: i64,
    pub event_set_id: String,
    pub operation: String,
    pub author_id: String,
    pub name: Option<String>,
    pub yomi: Option<String>,
    pub author_created_at: Option<OffsetDateTime>,
    pub author_updated_at: Option<OffsetDateTime>,
    pub changed_at: OffsetDateTime,
    pub extra: Option<Value>,
}

impl From<AuthorEvent> for AuthorEventDto {
    fn from(e: AuthorEvent) -> Self {
        Self {
            event_id: e.event_id,
            event_set_id: e.event_set_id.to_string(),
            operation: e.operation.as_str().to_string(),
            author_id: e.author_id.to_string(),
            name: e.name,
            yomi: e.yomi,
            author_created_at: e.author_created_at,
            author_updated_at: e.author_updated_at,
            changed_at: e.changed_at,
            extra: e.extra,
        }
    }
}
