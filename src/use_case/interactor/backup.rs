use async_trait::async_trait;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    domain::{entity::user::UserId, repository::backup_repository::BackupRepository},
    use_case::{
        dto::backup::{
            BACKUP_VERSION_V1, CURRENT_BACKUP_FORMAT, CurrentBackupV1, FULL_BACKUP_FORMAT,
            FullBackupV1,
        },
        error::UseCaseError,
        traits::backup::BackupUseCase,
    },
};

#[derive(Debug, Clone)]
pub struct BackupInteractor<BR> {
    repository: BR,
}

impl<BR> BackupInteractor<BR> {
    pub fn new(repository: BR) -> Self {
        Self { repository }
    }
}

fn now() -> Result<String, UseCaseError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| UseCaseError::Unexpected(error.to_string()))
}

#[async_trait]
impl<BR: BackupRepository> BackupUseCase for BackupInteractor<BR> {
    async fn export_current(&self, user_id: &str) -> Result<CurrentBackupV1, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        Ok(CurrentBackupV1 {
            format: CURRENT_BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION_V1,
            exported_at: now()?,
            data: self.repository.export_current(&user_id).await?,
        })
    }

    async fn export_full(&self, user_id: &str) -> Result<FullBackupV1, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let (data, history) = self.repository.export_full(&user_id).await?;
        Ok(FullBackupV1 {
            format: FULL_BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION_V1,
            exported_at: now()?,
            data,
            history,
        })
    }

    async fn restore_current(&self, user_id: &str, value: Value) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let backup = CurrentBackupV1::parse(value)
            .map_err(|error| UseCaseError::Validation(error.to_string()))?;
        self.repository
            .restore_current(&user_id, &backup.data)
            .await?;
        Ok(())
    }

    async fn restore_full(&self, user_id: &str, value: Value) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let backup = FullBackupV1::parse(value)
            .map_err(|error| UseCaseError::Validation(error.to_string()))?;
        self.repository
            .restore_full(&user_id, &backup.data, &backup.history)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        domain::repository::backup_repository::MockBackupRepository,
        use_case::dto::backup::CurrentBackupDataV1,
    };

    fn empty_current() -> Value {
        json!({
            "format": CURRENT_BACKUP_FORMAT,
            "version": 1,
            "exportedAt": "2026-08-26T00:00:00Z",
            "data": {"authors": [], "books": []}
        })
    }

    #[tokio::test]
    async fn restore_uses_authenticated_user_id() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_restore_current()
            .withf(|user_id, data| {
                user_id.as_str() == "authenticated-user"
                    && data.authors.is_empty()
                    && data.books.is_empty()
            })
            .once()
            .returning(|_, _| Ok(()));
        let interactor = BackupInteractor::new(repository);

        interactor
            .restore_current("authenticated-user", empty_current())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn invalid_backup_is_rejected_before_repository_write() {
        let mut repository = MockBackupRepository::new();
        repository.expect_restore_current().never();
        let interactor = BackupInteractor::new(repository);
        let mut invalid = empty_current();
        invalid["version"] = json!(2);

        assert!(matches!(
            interactor
                .restore_current("authenticated-user", invalid)
                .await,
            Err(UseCaseError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn export_builds_v1_envelope() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_export_current()
            .withf(|user_id| user_id.as_str() == "authenticated-user")
            .once()
            .returning(|_| {
                Ok(CurrentBackupDataV1 {
                    authors: vec![],
                    books: vec![],
                })
            });
        let interactor = BackupInteractor::new(repository);

        let backup = interactor
            .export_current("authenticated-user")
            .await
            .unwrap();
        assert_eq!(backup.format, CURRENT_BACKUP_FORMAT);
        assert_eq!(backup.version, BACKUP_VERSION_V1);
    }
}
