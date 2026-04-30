use serde_json::Value;
use time::OffsetDateTime;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::entity::{
        author::AuthorId,
        book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
        event_set::EventSetId,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventOperation {
    Create,
    Update,
    Delete,
    Restore,
    Snapshot,
}

impl EventOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventOperation::Create => "create",
            EventOperation::Update => "update",
            EventOperation::Delete => "delete",
            EventOperation::Restore => "restore",
            EventOperation::Snapshot => "snapshot",
        }
    }
}

impl TryFrom<&str> for EventOperation {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "create" => Ok(EventOperation::Create),
            "update" => Ok(EventOperation::Update),
            "delete" => Ok(EventOperation::Delete),
            "restore" => Ok(EventOperation::Restore),
            "snapshot" => Ok(EventOperation::Snapshot),
            _ => Err(format!("Unknown event operation: {}", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BookEvent {
    pub event_id: i64,
    pub event_set_id: EventSetId,
    pub operation: EventOperation,
    pub book_id: BookId,
    // Some for create/update/restore/snapshot; None for delete:
    pub title: Option<BookTitle>,
    pub author_ids: Vec<AuthorId>,
    pub isbn: Option<Isbn>,
    pub read: Option<ReadFlag>,
    pub owned: Option<OwnedFlag>,
    pub priority: Option<Priority>,
    pub format: Option<BookFormat>,
    pub store: Option<BookStore>,
    pub book_created_at: Option<OffsetDateTime>,
    pub book_updated_at: Option<OffsetDateTime>,
    pub changed_at: OffsetDateTime,
    // Operation-specific extra data (e.g. source_event_id for restore)
    pub extra: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct AuthorEvent {
    pub event_id: i64,
    pub event_set_id: EventSetId,
    pub operation: EventOperation,
    pub author_id: AuthorId,
    // Some for create/update/restore/snapshot; None for delete:
    pub name: Option<String>,
    pub yomi: Option<String>,
    pub author_created_at: Option<OffsetDateTime>,
    pub author_updated_at: Option<OffsetDateTime>,
    pub changed_at: OffsetDateTime,
    // Operation-specific extra data (e.g. source_event_id for restore)
    pub extra: Option<Value>,
}
