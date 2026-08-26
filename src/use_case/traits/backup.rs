use async_trait::async_trait;
use serde_json::Value;

use crate::use_case::{
    dto::backup::{FullBackupV1, StateBackupV1},
    error::UseCaseError,
};

#[async_trait]
pub trait BackupUseCase: Send + Sync + 'static {
    async fn export_state(&self, user_id: &str) -> Result<StateBackupV1, UseCaseError>;
    async fn export_full(&self, user_id: &str) -> Result<FullBackupV1, UseCaseError>;
    async fn restore_state(&self, user_id: &str, value: Value) -> Result<(), UseCaseError>;
    async fn restore_full(&self, user_id: &str, value: Value) -> Result<(), UseCaseError>;
}
