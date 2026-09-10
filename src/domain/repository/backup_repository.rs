use async_trait::async_trait;

use crate::{
    domain::{entity::user::UserId, error::DomainError},
    use_case::dto::backup::{BackupData, BackupScope},
};

#[async_trait]
pub trait BackupRepository: Send + Sync + Clone + 'static {
    async fn export(&self, user_id: &UserId, scope: BackupScope)
    -> Result<BackupData, DomainError>;
}
