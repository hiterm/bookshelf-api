use crate::{
    domain::{
        entity::user::UserId, error::DomainError, repository::backup_repository::BackupRepository,
    },
    use_case::dto::backup::{BackupData, BackupScope},
};

#[derive(Debug, Clone)]
pub struct BackupQueryInteractor<R: BackupRepository> {
    repository: R,
}

impl<R: BackupRepository> BackupQueryInteractor<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn export(
        &self,
        user_id: &UserId,
        scope: BackupScope,
    ) -> Result<BackupData, DomainError> {
        self.repository.export(user_id, scope).await
    }
}
