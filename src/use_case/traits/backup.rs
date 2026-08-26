use async_trait::async_trait;
use serde_json::Value;

use crate::use_case::{
    dto::backup::{BackupValidationResponse, FullBackupV1, SnapshotBackupV1},
    error::UseCaseError,
};

#[async_trait]
pub trait BackupUseCase: Send + Sync + 'static {
    async fn export_snapshot(&self, user_id: &str) -> Result<SnapshotBackupV1, UseCaseError>;
    async fn export_full(&self, user_id: &str) -> Result<FullBackupV1, UseCaseError>;
    async fn validate_snapshot(
        &self,
        user_id: &str,
        value: Value,
    ) -> Result<BackupValidationResponse, UseCaseError>;
    async fn restore_snapshot(&self, user_id: &str, value: Value) -> Result<(), UseCaseError>;
}
