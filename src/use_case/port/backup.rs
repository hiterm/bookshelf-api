use async_trait::async_trait;
use mockall::automock;
use time::{Date, OffsetDateTime};

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{entity::user::UserId, error::DomainError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupAuthorProjection {
    pub id: String,
    pub name: String,
    pub yomi: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBookSnapshotProjection {
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub purchase_date: Option<Date>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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
    pub recorded_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupAuthorSnapshotProjection {
    pub name: String,
    pub yomi: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupAuthorRevisionProjection {
    pub author_id: String,
    pub revision_number: i32,
    pub snapshot: BackupAuthorSnapshotProjection,
    pub recorded_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBookChangeProjection {
    pub book_id: String,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupAuthorChangeProjection {
    pub author_id: String,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackupChangesProjection {
    pub books: Vec<BackupBookChangeProjection>,
    pub authors: Vec<BackupAuthorChangeProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupOperationProjection {
    pub id: String,
    pub operation_type: String,
    pub detail: Option<serde_json::Value>,
    pub undo_of_operation_id: Option<String>,
    pub created_at: OffsetDateTime,
    pub changes: BackupChangesProjection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupHistoryProjection {
    pub operations: Vec<BackupOperationProjection>,
    pub book_revisions: Vec<BackupBookRevisionProjection>,
    pub author_revisions: Vec<BackupAuthorRevisionProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupSnapshotProjection {
    pub authors: Vec<BackupAuthorProjection>,
    pub books: Vec<BackupBookProjection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupFullProjection {
    pub authors: Vec<BackupAuthorProjection>,
    pub books: Vec<BackupBookProjection>,
    pub history: BackupHistoryProjection,
}

#[automock]
#[async_trait]
pub trait BackupQueryPort: Send + Sync + 'static {
    async fn snapshot(&self, user_id: &UserId) -> Result<BackupSnapshotProjection, DomainError>;

    async fn full(&self, user_id: &UserId) -> Result<BackupFullProjection, DomainError>;
}
