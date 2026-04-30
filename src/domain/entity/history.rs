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
pub enum HistoryOperation {
    Create,
    Update,
    Delete,
}

impl HistoryOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            HistoryOperation::Create => "create",
            HistoryOperation::Update => "update",
            HistoryOperation::Delete => "delete",
        }
    }
}

impl TryFrom<&str> for HistoryOperation {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "create" => Ok(HistoryOperation::Create),
            "update" => Ok(HistoryOperation::Update),
            "delete" => Ok(HistoryOperation::Delete),
            _ => Err(format!("Unknown history operation: {}", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BookEvent {
    pub event_id: i64,
    pub event_set_id: EventSetId,
    pub operation: HistoryOperation,
    pub book_id: BookId,
    // Some for create/update; None for delete:
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
}

#[derive(Debug, Clone)]
pub struct AuthorEvent {
    pub event_id: i64,
    pub event_set_id: EventSetId,
    pub operation: HistoryOperation,
    pub author_id: AuthorId,
    // Some for create/update; None for delete:
    pub name: Option<String>,
    pub yomi: Option<String>,
    pub author_created_at: Option<OffsetDateTime>,
    pub author_updated_at: Option<OffsetDateTime>,
    pub changed_at: OffsetDateTime,
}
