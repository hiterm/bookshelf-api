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

/// TODO: Migrate other event_set operations (create_book, update_book, etc.)
/// from hard-coded strings in book_repository.rs and author_repository.rs.
/// See: https://github.com/hiterm/bookshelf-api/pull/225#issuecomment-4466306766
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSetOperation {
    ImportBooks,
}

impl EventSetOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSetOperation::ImportBooks => "import_books",
        }
    }
}

impl TryFrom<&str> for EventSetOperation {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "import_books" => Ok(EventSetOperation::ImportBooks),
            _ => Err(format!("Unknown event set operation: {}", value)),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{EventOperation, EventSetOperation};

    #[test]
    fn event_operation_round_trip() {
        let variants = [
            EventOperation::Create,
            EventOperation::Update,
            EventOperation::Delete,
            EventOperation::Restore,
            EventOperation::Snapshot,
        ];
        for variant in &variants {
            let s = variant.as_str();
            let back = EventOperation::try_from(s).expect("round-trip failed");
            assert_eq!(&back, variant, "round-trip mismatch for {:?}", variant);
        }
    }

    #[test]
    fn event_operation_unknown_returns_err() {
        assert!(EventOperation::try_from("invalid").is_err());
    }

    #[test]
    fn event_set_operation_as_str() {
        assert_eq!(EventSetOperation::ImportBooks.as_str(), "import_books");
    }

    #[test]
    fn event_set_operation_roundtrip() {
        let s = EventSetOperation::ImportBooks.as_str();
        let back = EventSetOperation::try_from(s).expect("round-trip failed");
        assert_eq!(back, EventSetOperation::ImportBooks);
    }

    #[test]
    fn event_set_operation_unknown_returns_err() {
        assert!(EventSetOperation::try_from("invalid").is_err());
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
