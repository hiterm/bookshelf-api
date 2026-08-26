use async_trait::async_trait;
use mockall::automock;

use crate::{
    domain::{entity::user::UserId, error::DomainError},
    use_case::dto::backup::{BackupHistoryV1, CurrentBackupDataV1},
};

#[automock]
#[async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn export_current(&self, user_id: &UserId) -> Result<CurrentBackupDataV1, DomainError>;
    async fn export_full(
        &self,
        user_id: &UserId,
    ) -> Result<(CurrentBackupDataV1, BackupHistoryV1), DomainError>;
    async fn restore_current(
        &self,
        user_id: &UserId,
        data: &CurrentBackupDataV1,
    ) -> Result<(), DomainError>;
    async fn restore_full(
        &self,
        user_id: &UserId,
        data: &CurrentBackupDataV1,
        history: &BackupHistoryV1,
    ) -> Result<(), DomainError>;
}
