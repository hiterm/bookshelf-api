use async_trait::async_trait;
use mockall::automock;

use crate::{
    domain::{entity::user::UserId, error::DomainError},
    use_case::dto::backup::{BackupHistoryV1, SnapshotBackupDataV1},
};

#[automock]
#[async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn export_snapshot(&self, user_id: &UserId) -> Result<SnapshotBackupDataV1, DomainError>;
    async fn export_full(
        &self,
        user_id: &UserId,
    ) -> Result<(SnapshotBackupDataV1, BackupHistoryV1), DomainError>;
    async fn restore_snapshot(
        &self,
        user_id: &UserId,
        data: &SnapshotBackupDataV1,
    ) -> Result<(), DomainError>;
}
