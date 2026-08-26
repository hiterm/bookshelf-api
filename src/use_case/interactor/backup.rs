use async_trait::async_trait;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    domain::{entity::user::UserId, repository::backup_repository::BackupRepository},
    use_case::{
        dto::backup::{
            BACKUP_VERSION_V1, BackupValidationResponse, FULL_BACKUP_FORMAT, FullBackupV1,
            SNAPSHOT_BACKUP_FORMAT, SnapshotBackupV1, validate_snapshot_backup,
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
    async fn export_snapshot(&self, user_id: &str) -> Result<SnapshotBackupV1, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        Ok(SnapshotBackupV1 {
            format: SNAPSHOT_BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION_V1,
            exported_at: now()?,
            data: self.repository.export_snapshot(&user_id).await?,
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

    async fn validate_snapshot(
        &self,
        user_id: &str,
        value: Value,
    ) -> Result<BackupValidationResponse, UseCaseError> {
        UserId::new(user_id.to_string())?;
        Ok(validate_snapshot_backup(value).response)
    }

    async fn restore_snapshot(&self, user_id: &str, value: Value) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let validation = validate_snapshot_backup(value);
        let backup = validation
            .backup
            .ok_or(UseCaseError::BackupValidation(validation.response))?;
        self.repository
            .restore_snapshot(&user_id, &backup.data)
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
        use_case::dto::backup::{BackupHistoryV1, SnapshotBackupDataV1},
    };

    fn empty_snapshot() -> Value {
        json!({
            "format": SNAPSHOT_BACKUP_FORMAT,
            "version": 1,
            "exportedAt": "2026-08-26T00:00:00Z",
            "data": {"authors": [], "books": []}
        })
    }

    #[tokio::test]
    async fn restore_uses_authenticated_user_id() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_restore_snapshot()
            .withf(|user_id, data| {
                user_id.as_str() == "authenticated-user"
                    && data.authors.is_empty()
                    && data.books.is_empty()
            })
            .once()
            .returning(|_, _| Ok(()));
        let interactor = BackupInteractor::new(repository);

        interactor
            .restore_snapshot("authenticated-user", empty_snapshot())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn invalid_backup_is_rejected_before_repository_write() {
        let mut repository = MockBackupRepository::new();
        repository.expect_restore_snapshot().never();
        let interactor = BackupInteractor::new(repository);
        let mut invalid = empty_snapshot();
        invalid["version"] = json!(2);

        assert!(matches!(
            interactor
                .restore_snapshot("authenticated-user", invalid)
                .await,
            Err(UseCaseError::BackupValidation(_))
        ));
    }

    #[tokio::test]
    async fn validate_snapshot_is_read_only() {
        let mut repository = MockBackupRepository::new();
        repository.expect_restore_snapshot().never();
        let interactor = BackupInteractor::new(repository);

        let response = interactor
            .validate_snapshot("authenticated-user", empty_snapshot())
            .await
            .unwrap();
        assert!(response.valid);
        assert_eq!(response.summary.unwrap().books, 0);
    }

    #[tokio::test]
    async fn export_builds_v1_envelope() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_export_snapshot()
            .withf(|user_id| user_id.as_str() == "authenticated-user")
            .once()
            .returning(|_| {
                Ok(SnapshotBackupDataV1 {
                    authors: vec![],
                    books: vec![],
                })
            });
        let interactor = BackupInteractor::new(repository);

        let backup = interactor
            .export_snapshot("authenticated-user")
            .await
            .unwrap();
        assert_eq!(backup.format, SNAPSHOT_BACKUP_FORMAT);
        assert_eq!(backup.version, BACKUP_VERSION_V1);
    }

    #[tokio::test]
    async fn export_full_builds_v1_envelope_with_repository_data() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_export_full()
            .withf(|user_id| user_id.as_str() == "authenticated-user")
            .once()
            .returning(|_| {
                Ok((
                    SnapshotBackupDataV1 {
                        authors: vec![],
                        books: vec![],
                    },
                    BackupHistoryV1 {
                        event_sets: vec![],
                        book_events: vec![],
                        author_events: vec![],
                    },
                ))
            });
        let interactor = BackupInteractor::new(repository);

        let backup = interactor.export_full("authenticated-user").await.unwrap();
        assert_eq!(backup.format, FULL_BACKUP_FORMAT);
        assert_eq!(backup.version, BACKUP_VERSION_V1);
        assert!(backup.data.books.is_empty());
        assert!(backup.history.event_sets.is_empty());
        let value = serde_json::to_value(backup).unwrap();
        assert!(value.get("exportedAt").is_some());
        assert!(value["history"].get("eventSets").is_some());
        assert!(value["history"].get("bookEvents").is_some());
        assert!(value["history"].get("authorEvents").is_some());
    }
}
