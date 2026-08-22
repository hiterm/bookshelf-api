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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventId(i64);

impl EventId {
    pub fn value(self) -> i64 {
        self.0
    }
}

impl From<i64> for EventId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventOperation {
    Create,
    Update,
    Delete,
    Restore,
    Snapshot,
    MergeAsDestination,
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
    MergeAuthor,
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
            EventSetOperation::MergeAuthor => "merge_author",
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
            "merge_author" => Ok(EventSetOperation::MergeAuthor),
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
            EventOperation::MergeAsDestination => "merge_as_destination",
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
            "merge_as_destination" => Ok(EventOperation::MergeAsDestination),
            _ => Err(format!("Unknown event operation: {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EventId, EventOperation, EventSetOperation};

    #[test]
    fn event_id_round_trips_database_value() {
        let event_id = EventId::from(42);

        assert_eq!(event_id.value(), 42);
        assert_eq!(event_id.to_string(), "42");
    }

    #[test]
    fn event_operation_round_trip() {
        let variants = [
            EventOperation::Create,
            EventOperation::Update,
            EventOperation::Delete,
            EventOperation::Restore,
            EventOperation::Snapshot,
            EventOperation::MergeAsDestination,
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
        assert_eq!(EventSetOperation::MergeAuthor.as_str(), "merge_author");
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
            EventSetOperation::MergeAuthor,
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

#[derive(Debug, Clone, PartialEq)]
pub struct NewAuthorEvent {
    pub operation: EventOperation,
    pub author_id: AuthorId,
    pub name: Option<String>,
    pub yomi: Option<String>,
    pub author_created_at: Option<OffsetDateTime>,
    pub author_updated_at: Option<OffsetDateTime>,
    pub extra: Option<Value>,
}

impl NewAuthorEvent {
    pub fn merge_as_destination(author_id: AuthorId, source_author_id: &AuthorId) -> Self {
        Self {
            operation: EventOperation::MergeAsDestination,
            author_id,
            name: None,
            yomi: None,
            author_created_at: None,
            author_updated_at: None,
            extra: Some(serde_json::json!({
                "version": 1,
                "source_author_id": source_author_id.to_string(),
            })),
        }
    }
}
