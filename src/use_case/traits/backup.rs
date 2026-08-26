use async_trait::async_trait;
use serde_json::Value;

use crate::use_case::{
    dto::backup::{CurrentBackupV1, FullBackupV1},
    error::UseCaseError,
};

#[async_trait]
pub trait BackupUseCase: Send + Sync + 'static {
    async fn export_current(&self, user_id: &str) -> Result<CurrentBackupV1, UseCaseError>;
    async fn export_full(&self, user_id: &str) -> Result<FullBackupV1, UseCaseError>;
    async fn restore_current(&self, user_id: &str, value: Value) -> Result<(), UseCaseError>;
    async fn restore_full(&self, user_id: &str, value: Value) -> Result<(), UseCaseError>;
}
