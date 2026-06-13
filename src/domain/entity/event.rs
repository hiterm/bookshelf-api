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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSetOperation {
    CreateBook,
    UpdateBook,
    DeleteBook,
    RestoreBook,
    CreateAuthor,
    UpdateAuthor,
    DeleteAuthor,
    RestoreAuthor,
    ImportBooks,
    SnapshotAll,
}

impl EventSetOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSetOperation::CreateBook => "create_book",
            EventSetOperation::UpdateBook => "update_book",
            EventSetOperation::DeleteBook => "delete_book",
            EventSetOperation::RestoreBook => "restore_book",
            EventSetOperation::CreateAuthor => "create_author",
            EventSetOperation::UpdateAuthor => "update_author",
            EventSetOperation::DeleteAuthor => "delete_author",
            EventSetOperation::RestoreAuthor => "restore_author",
            EventSetOperation::ImportBooks => "import_books",
            EventSetOperation::SnapshotAll => "snapshot_all",
        }
    }
}

impl TryFrom<&str> for EventSetOperation {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "create_book" => Ok(EventSetOperation::CreateBook),
            "update_book" => Ok(EventSetOperation::UpdateBook),
            "delete_book" => Ok(EventSetOperation::DeleteBook),
            "restore_book" => Ok(EventSetOperation::RestoreBook),
            "create_author" => Ok(EventSetOperation::CreateAuthor),
            "update_author" => Ok(EventSetOperation::UpdateAuthor),
            "delete_author" => Ok(EventSetOperation::DeleteAuthor),
            "restore_author" => Ok(EventSetOperation::RestoreAuthor),
            "import_books" => Ok(EventSetOperation::ImportBooks),
            "snapshot_all" => Ok(EventSetOperation::SnapshotAll),
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
        assert_eq!(EventSetOperation::CreateBook.as_str(), "create_book");
        assert_eq!(EventSetOperation::UpdateBook.as_str(), "update_book");
        assert_eq!(EventSetOperation::DeleteBook.as_str(), "delete_book");
        assert_eq!(EventSetOperation::RestoreBook.as_str(), "restore_book");
        assert_eq!(EventSetOperation::CreateAuthor.as_str(), "create_author");
        assert_eq!(EventSetOperation::UpdateAuthor.as_str(), "update_author");
        assert_eq!(EventSetOperation::DeleteAuthor.as_str(), "delete_author");
        assert_eq!(EventSetOperation::RestoreAuthor.as_str(), "restore_author");
        assert_eq!(EventSetOperation::ImportBooks.as_str(), "import_books");
        assert_eq!(EventSetOperation::SnapshotAll.as_str(), "snapshot_all");
    }

    #[test]
    fn event_set_operation_roundtrip() {
        let variants = [
            EventSetOperation::CreateBook,
            EventSetOperation::UpdateBook,
            EventSetOperation::DeleteBook,
            EventSetOperation::RestoreBook,
            EventSetOperation::CreateAuthor,
            EventSetOperation::UpdateAuthor,
            EventSetOperation::DeleteAuthor,
            EventSetOperation::RestoreAuthor,
            EventSetOperation::ImportBooks,
            EventSetOperation::SnapshotAll,
        ];
        for variant in &variants {
            let s = variant.as_str();
            let back = EventSetOperation::try_from(s).expect("round-trip failed");
            assert_eq!(&back, variant, "round-trip mismatch for {:?}", variant);
        }
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
