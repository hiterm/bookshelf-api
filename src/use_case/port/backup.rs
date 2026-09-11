use async_trait::async_trait;
use mockall::automock;

use crate::domain::{entity::user::UserId, error::DomainError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupAuthorProjection {
    pub id: String,
    pub name: String,
    pub yomi: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBookSnapshotProjection {
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: String,
    pub store: String,
    pub purchase_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBookProjection {
    pub id: String,
    pub snapshot: BackupBookSnapshotProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBookRevisionProjection {
    pub book_id: String,
    pub revision_number: i32,
    pub snapshot: BackupBookSnapshotProjection,
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupAuthorSnapshotProjection {
    pub name: String,
    pub yomi: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupAuthorRevisionProjection {
    pub author_id: String,
    pub revision_number: i32,
    pub snapshot: BackupAuthorSnapshotProjection,
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupEntityChangeProjection {
    pub book_id: Option<String>,
    pub author_id: Option<String>,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackupChangesProjection {
    pub books: Vec<BackupEntityChangeProjection>,
    pub authors: Vec<BackupEntityChangeProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupOperationProjection {
    pub id: String,
    pub operation_type: String,
    pub detail: Option<serde_json::Value>,
    pub undo_of_operation_id: Option<String>,
    pub created_at: String,
    pub changes: BackupChangesProjection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupHistoryProjection {
    pub operations: Vec<BackupOperationProjection>,
    pub book_revisions: Vec<BackupBookRevisionProjection>,
    pub author_revisions: Vec<BackupAuthorRevisionProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupProjection {
    pub authors: Vec<BackupAuthorProjection>,
    pub books: Vec<BackupBookProjection>,
    pub history: Option<BackupHistoryProjection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupQueryScope {
    Snapshot,
    Full,
}

#[automock]
#[async_trait]
pub trait BackupQueryPort: Send + Sync + 'static {
    async fn query(
        &self,
        user_id: &UserId,
        scope: BackupQueryScope,
    ) -> Result<BackupProjection, DomainError>;
}
