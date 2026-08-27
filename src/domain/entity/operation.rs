use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entity::{author::AuthorId, event::EventSetOperation, user::UserId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationId(Uuid);

impl OperationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for OperationId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for OperationId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value)
            .map(Self)
            .map_err(|error| error.to_string())
    }
}

impl std::fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0.hyphenated())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationType {
    Baseline,
    CreateBook,
    UpdateBook,
    DeleteBook,
    RestoreBook,
    CreateAuthor,
    UpdateAuthor,
    DeleteAuthor,
    RestoreAuthor,
    ImportBooks,
    MergeAuthor,
    RestoreBackup,
    Undo,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::CreateBook => "create_book",
            Self::UpdateBook => "update_book",
            Self::DeleteBook => "delete_book",
            Self::RestoreBook => "restore_book",
            Self::CreateAuthor => "create_author",
            Self::UpdateAuthor => "update_author",
            Self::DeleteAuthor => "delete_author",
            Self::RestoreAuthor => "restore_author",
            Self::ImportBooks => "import_books",
            Self::MergeAuthor => "merge_author",
            Self::RestoreBackup => "restore_backup",
            Self::Undo => "undo",
        }
    }
}

impl TryFrom<&str> for OperationType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "baseline" => Ok(Self::Baseline),
            "create_book" => Ok(Self::CreateBook),
            "update_book" => Ok(Self::UpdateBook),
            "delete_book" => Ok(Self::DeleteBook),
            "restore_book" => Ok(Self::RestoreBook),
            "create_author" => Ok(Self::CreateAuthor),
            "update_author" => Ok(Self::UpdateAuthor),
            "delete_author" => Ok(Self::DeleteAuthor),
            "restore_author" => Ok(Self::RestoreAuthor),
            "import_books" => Ok(Self::ImportBooks),
            "merge_author" => Ok(Self::MergeAuthor),
            "restore_backup" => Ok(Self::RestoreBackup),
            "undo" => Ok(Self::Undo),
            _ => Err(format!("Unknown operation type: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OperationDetail {
    MergeAuthor {
        source_author_id: Uuid,
        destination_author_id: Uuid,
    },
    RestoreBook {
        source_revision_number: i32,
    },
    RestoreAuthor {
        source_revision_number: i32,
    },
    ImportBooks {
        imported_count: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewOperation {
    pub operation_type: OperationType,
    pub detail: Option<OperationDetail>,
    pub undo_of_operation_id: Option<OperationId>,
}

impl NewOperation {
    pub fn simple(operation_type: OperationType) -> Self {
        Self {
            operation_type,
            detail: None,
            undo_of_operation_id: None,
        }
    }

    pub fn merge_author(source: &AuthorId, destination: &AuthorId) -> Self {
        Self {
            operation_type: OperationType::MergeAuthor,
            detail: Some(OperationDetail::MergeAuthor {
                source_author_id: source.to_uuid(),
                destination_author_id: destination.to_uuid(),
            }),
            undo_of_operation_id: None,
        }
    }

    pub fn import_books(imported_count: usize) -> Result<Self, String> {
        let imported_count = u32::try_from(imported_count)
            .map_err(|_| "imported book count exceeds u32".to_string())?;
        Ok(Self {
            operation_type: OperationType::ImportBooks,
            detail: Some(OperationDetail::ImportBooks { imported_count }),
            undo_of_operation_id: None,
        })
    }
}

impl From<EventSetOperation> for NewOperation {
    fn from(value: EventSetOperation) -> Self {
        let operation_type = match value {
            EventSetOperation::CreateBook => OperationType::CreateBook,
            EventSetOperation::UpdateBook => OperationType::UpdateBook,
            EventSetOperation::DeleteBook => OperationType::DeleteBook,
            EventSetOperation::RestoreBook => OperationType::RestoreBook,
            EventSetOperation::CreateAuthor => OperationType::CreateAuthor,
            EventSetOperation::UpdateAuthor => OperationType::UpdateAuthor,
            EventSetOperation::DeleteAuthor => OperationType::DeleteAuthor,
            EventSetOperation::RestoreAuthor => OperationType::RestoreAuthor,
            EventSetOperation::ImportBooks => OperationType::ImportBooks,
            EventSetOperation::SnapshotAll => OperationType::Baseline,
            EventSetOperation::MergeAuthor => OperationType::MergeAuthor,
        };
        Self::simple(operation_type)
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub id: OperationId,
    pub user_id: UserId,
    pub operation_type: OperationType,
    pub detail: Option<OperationDetail>,
    pub undo_of_operation_id: Option<OperationId>,
    pub created_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::{OperationDetail, OperationId, OperationType};

    #[test]
    fn operation_id_round_trips_uuid_text() {
        let text = "c6ea22c8-7b70-470c-a713-c7aade5693bd";
        let id = OperationId::try_from(text).expect("valid operation id");

        assert_eq!(id.to_string(), text);
    }

    #[test]
    fn operation_types_round_trip_database_values() {
        let variants = [
            OperationType::Baseline,
            OperationType::CreateBook,
            OperationType::UpdateBook,
            OperationType::DeleteBook,
            OperationType::RestoreBook,
            OperationType::CreateAuthor,
            OperationType::UpdateAuthor,
            OperationType::DeleteAuthor,
            OperationType::RestoreAuthor,
            OperationType::ImportBooks,
            OperationType::MergeAuthor,
            OperationType::RestoreBackup,
            OperationType::Undo,
        ];

        for variant in variants {
            assert_eq!(OperationType::try_from(variant.as_str()), Ok(variant));
        }
    }

    #[test]
    fn operation_detail_has_typed_json_shape() {
        let detail = OperationDetail::RestoreBook {
            source_revision_number: 3,
        };

        assert_eq!(
            serde_json::to_value(detail).expect("serializable detail"),
            serde_json::json!({"type": "restore_book", "source_revision_number": 3})
        );
    }
}
