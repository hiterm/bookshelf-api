use time::OffsetDateTime;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::entity::{
        author::AuthorId,
        book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
        change_set::ChangeSetId,
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
pub struct BookHistory {
    pub history_id: i64,
    pub change_set_id: ChangeSetId,
    pub operation: HistoryOperation,
    pub book_id: BookId,
    pub title: BookTitle,
    pub author_ids: Vec<AuthorId>,
    pub isbn: Isbn,
    pub read: ReadFlag,
    pub owned: OwnedFlag,
    pub priority: Priority,
    pub format: BookFormat,
    pub store: BookStore,
    pub book_created_at: OffsetDateTime,
    pub book_updated_at: OffsetDateTime,
    pub changed_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct AuthorHistory {
    pub history_id: i64,
    pub change_set_id: ChangeSetId,
    pub operation: HistoryOperation,
    pub author_id: AuthorId,
    pub name: String,
    pub yomi: String,
    pub author_created_at: OffsetDateTime,
    pub author_updated_at: OffsetDateTime,
    pub changed_at: OffsetDateTime,
}
