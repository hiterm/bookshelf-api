use async_trait::async_trait;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    domain::{entity::user::UserId, repository::backup_repository::BackupRepository},
    use_case::{
        dto::backup::{
            BACKUP_VERSION_V1, FULL_BACKUP_FORMAT, FullBackupV1, STATE_BACKUP_FORMAT, StateBackupV1,
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
    async fn export_state(&self, user_id: &str) -> Result<StateBackupV1, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        Ok(StateBackupV1 {
            format: STATE_BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION_V1,
            exported_at: now()?,
            data: self.repository.export_state(&user_id).await?,
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

    async fn restore_state(&self, user_id: &str, value: Value) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let backup = StateBackupV1::parse(value)
            .map_err(|error| UseCaseError::Validation(error.to_string()))?;
        self.repository
            .restore_state(&user_id, &backup.data)
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
        use_case::dto::backup::{BackupHistoryV1, StateBackupDataV1},
    };

    fn empty_state() -> Value {
        json!({
            "format": STATE_BACKUP_FORMAT,
            "version": 1,
            "exportedAt": "2026-08-26T00:00:00Z",
            "data": {"authors": [], "books": []}
        })
    }

    fn empty_full() -> Value {
        json!({
            "format": FULL_BACKUP_FORMAT,
            "version": 1,
            "exportedAt": "2026-08-26T00:00:00Z",
            "data": {"authors": [], "books": []},
            "history": {"eventSets": [], "bookEvents": [], "authorEvents": []}
        })
    }

    #[tokio::test]
    async fn restore_uses_authenticated_user_id() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_restore_state()
            .withf(|user_id, data| {
                user_id.as_str() == "authenticated-user"
                    && data.authors.is_empty()
                    && data.books.is_empty()
            })
            .once()
            .returning(|_, _| Ok(()));
        let interactor = BackupInteractor::new(repository);

        interactor
            .restore_state("authenticated-user", empty_state())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn invalid_backup_is_rejected_before_repository_write() {
        let mut repository = MockBackupRepository::new();
        repository.expect_restore_state().never();
        let interactor = BackupInteractor::new(repository);
        let mut invalid = empty_state();
        invalid["version"] = json!(2);

        assert!(matches!(
            interactor
                .restore_state("authenticated-user", invalid)
                .await,
            Err(UseCaseError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn export_builds_v1_envelope() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_export_state()
            .withf(|user_id| user_id.as_str() == "authenticated-user")
            .once()
            .returning(|_| {
                Ok(StateBackupDataV1 {
                    authors: vec![],
                    books: vec![],
                })
            });
        let interactor = BackupInteractor::new(repository);

        let backup = interactor.export_state("authenticated-user").await.unwrap();
        assert_eq!(backup.format, STATE_BACKUP_FORMAT);
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
                    StateBackupDataV1 {
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
        assert!(backup.data.books.is_empty());
        assert!(backup.history.event_sets.is_empty());
    }

    #[tokio::test]
    async fn invalid_full_backup_is_rejected_before_repository_write() {
        let mut repository = MockBackupRepository::new();
        repository.expect_restore_full().never();
        let interactor = BackupInteractor::new(repository);
        let mut invalid = empty_full();
        invalid["version"] = json!(2);

        assert!(matches!(
            interactor.restore_full("authenticated-user", invalid).await,
            Err(UseCaseError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn restore_full_passes_data_and_history_to_repository() {
        let mut repository = MockBackupRepository::new();
        repository
            .expect_restore_full()
            .withf(|user_id, data, history| {
                user_id.as_str() == "authenticated-user"
                    && data.authors.is_empty()
                    && history.event_sets.is_empty()
            })
            .once()
            .returning(|_, _, _| Ok(()));
        let interactor = BackupInteractor::new(repository);

        interactor
            .restore_full("authenticated-user", empty_full())
            .await
            .unwrap();
    }
}
