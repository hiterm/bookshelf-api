use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName, AuthorUpdate, validate_author_yomi},
            event::EventSetOperation,
            user::UserId,
        },
        repository::{
            author_repository::AuthorRepository,
            transaction::{TransactionEventSet, TransactionManager},
        },
    },
    use_case::{
        dto::{
            author::{CreateAuthorDto, UpdateAuthorDto},
            mutation::{AuthorMutationResultDto, DeleteAuthorResultDto, MutationResultDto},
        },
        error::UseCaseError,
        traits::author::{CreateAuthorUseCase, DeleteAuthorUseCase, UpdateAuthorUseCase},
    },
};

pub struct CreateAuthorInteractor<AR, TM> {
    author_repository: AR,
    transaction_manager: TM,
}

impl<AR, TM> CreateAuthorInteractor<AR, TM> {
    pub fn new(author_repository: AR, transaction_manager: TM) -> Self {
        Self {
            author_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<AR, TM> CreateAuthorUseCase for CreateAuthorInteractor<AR, TM>
where
    TM: TransactionManager,
    AR: AuthorRepository<Transaction = TM::Transaction>,
{
    async fn create(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let uuid = Uuid::new_v4();
        let author_id = AuthorId::new(uuid);
        let author_name = AuthorName::new(author_data.name)?;
        let yomi = validate_author_yomi(author_data.yomi.unwrap_or_default())?;
        let now = OffsetDateTime::now_utc();
        let author = Author::new_with_yomi(author_id, author_name, yomi, now)?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::CreateAuthor)
            .await?;
        self.author_repository.create(&mut tx, &author).await?;
        let event_set_id = tx.event_set_id().hyphenated().to_string();
        self.transaction_manager.commit(tx).await?;

        Ok(MutationResultDto::new(author.into(), event_set_id))
    }
}

pub struct UpdateAuthorInteractor<AR, TM> {
    author_repository: AR,
    transaction_manager: TM,
}

impl<AR, TM> UpdateAuthorInteractor<AR, TM> {
    pub fn new(author_repository: AR, transaction_manager: TM) -> Self {
        Self {
            author_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<AR, TM> UpdateAuthorUseCase for UpdateAuthorInteractor<AR, TM>
where
    TM: TransactionManager,
    AR: AuthorRepository<Transaction = TM::Transaction>,
{
    async fn update(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_data.id.as_str())?;
        let author_name = AuthorName::new(author_data.name)?;
        let yomi = author_data.yomi.map(validate_author_yomi).transpose()?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::UpdateAuthor)
            .await?;
        let author = self
            .author_repository
            .find_by_id_with_tx(&mut tx, &user_id, &author_id)
            .await?;
        let mut author = match author {
            Some(author) => author,
            None => {
                return Err(UseCaseError::NotFound {
                    entity_type: "author",
                    entity_id: author_data.id,
                    user_id: user_id.into_string(),
                });
            }
        };

        author.update(
            AuthorUpdate {
                name: author_name,
                yomi,
            },
            OffsetDateTime::now_utc(),
        );

        self.author_repository.update(&mut tx, &author).await?;
        let event_set_id = tx.event_set_id().hyphenated().to_string();
        self.transaction_manager.commit(tx).await?;

        Ok(MutationResultDto::new(author.into(), event_set_id))
    }
}

pub struct DeleteAuthorInteractor<AR, TM> {
    author_repository: AR,
    transaction_manager: TM,
}

impl<AR, TM> DeleteAuthorInteractor<AR, TM> {
    pub fn new(author_repository: AR, transaction_manager: TM) -> Self {
        Self {
            author_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<AR, TM> DeleteAuthorUseCase for DeleteAuthorInteractor<AR, TM>
where
    TM: TransactionManager,
    AR: AuthorRepository<Transaction = TM::Transaction>,
{
    async fn delete(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<DeleteAuthorResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id_value = author_id.to_string();
        let author_id = AuthorId::try_from(author_id)?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::DeleteAuthor)
            .await?;
        self.author_repository.delete(&mut tx, &author_id).await?;
        let event_set_id = tx.event_set_id().hyphenated().to_string();
        self.transaction_manager.commit(tx).await?;

        Ok(MutationResultDto::new(author_id_value, event_set_id))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;
    use time::OffsetDateTime;

    use crate::{
        common::time::normalize_timestamp_for_persistence,
        domain::{
            entity::author::{Author, AuthorId, AuthorName},
            error::DomainError,
            repository::{
                author_repository::MockAuthorRepository, transaction::MockTransactionManager,
            },
        },
        use_case::{
            dto::author::{CreateAuthorDto, UpdateAuthorDto},
            error::UseCaseError,
            interactor::author::{
                CreateAuthorInteractor, DeleteAuthorInteractor, UpdateAuthorInteractor,
            },
            traits::author::{CreateAuthorUseCase, DeleteAuthorUseCase, UpdateAuthorUseCase},
        },
    };

    // A MockTransactionManager whose Transaction associated type is () and
    // whose begin/commit succeed, for interactors that reach the repository.
    fn make_transaction_manager() -> MockTransactionManager {
        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_commit().returning(|_| Ok(()));
        tm
    }

    #[tokio::test]
    async fn create_author_success() {
        // Given
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_create()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = CreateAuthorInteractor::new(author_repository, make_transaction_manager());
        let mut author_data = CreateAuthorDto::new("Test Author".to_string());
        author_data.yomi = Some("てすと・おーさー1".to_string());

        // When
        let before = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());
        let result = interactor.create("user1", author_data).await;
        let after = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());

        // Then
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.name, "Test Author");
        assert_eq!(dto.yomi, "てすと・おーさー1");
        assert_eq!(dto.created_at, dto.updated_at);
        assert!(dto.created_at >= before);
        assert!(dto.created_at <= after);
    }

    #[tokio::test]
    async fn create_author_fails_with_empty_name() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor =
            CreateAuthorInteractor::new(author_repository, MockTransactionManager::new());
        let author_data = CreateAuthorDto::new("".to_string());

        // When
        let result = interactor.create("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn create_author_fails_with_invalid_user_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor =
            CreateAuthorInteractor::new(author_repository, MockTransactionManager::new());
        let author_data = CreateAuthorDto::new("Test Author".to_string());

        // When
        let result = interactor.create("", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn create_author_fails_with_invalid_yomi() {
        let author_repository = MockAuthorRepository::new();
        let interactor =
            CreateAuthorInteractor::new(author_repository, MockTransactionManager::new());
        let mut author_data = CreateAuthorDto::new("Test Author".to_string());
        author_data.yomi = Some("テスト".to_string());

        let result = interactor.create("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_success() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let created_at = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap();
        let previous_updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        let existing_author = Author::new_with_timestamps(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Old Name".to_string()).unwrap(),
            "もとのよみ".to_string(),
            created_at,
            previous_updated_at,
        )
        .unwrap();

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .with(always(), always(), always())
            .returning(move |_, _, _| Ok(Some(existing_author.clone())));
        author_repository
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = UpdateAuthorInteractor::new(author_repository, make_transaction_manager());
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        // When
        let before = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());
        let result = interactor.update("user1", author_data).await;
        let after = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());

        // Then
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.yomi, "もとのよみ");
        assert_eq!(updated.created_at, created_at);
        assert!(updated.updated_at >= previous_updated_at);
        assert!(updated.updated_at >= before);
        assert!(updated.updated_at <= after);
    }

    #[tokio::test]
    async fn update_author_changes_yomi_when_provided() {
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let existing_author = Author::new_with_yomi(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Old Name".to_string()).unwrap(),
            "もとのよみ".to_string(),
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap();

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .return_once(move |_, _, _| Ok(Some(existing_author)));
        author_repository
            .expect_update()
            .withf(|_, author| author.yomi() == "")
            .returning(|_, _| Ok(()));

        let interactor = UpdateAuthorInteractor::new(author_repository, make_transaction_manager());
        let mut author_data =
            UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());
        author_data.yomi = Some(String::new());

        let result = interactor.update("user1", author_data).await.unwrap();

        assert_eq!(result.yomi, "");
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_yomi() {
        let author_repository = MockAuthorRepository::new();
        let interactor =
            UpdateAuthorInteractor::new(author_repository, MockTransactionManager::new());
        let mut author_data = UpdateAuthorDto::new(
            "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
            "New Name".to_string(),
        );
        author_data.yomi = Some("New Name".to_string());

        let result = interactor.update("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_not_found() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(None));

        let interactor = UpdateAuthorInteractor::new(author_repository, {
            let mut tm = MockTransactionManager::new();
            tm.expect_begin().returning(|_, _| Ok(()));
            tm
        });
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        // When
        let result = interactor.update("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_author_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor =
            UpdateAuthorInteractor::new(author_repository, MockTransactionManager::new());
        let author_data = UpdateAuthorDto::new("not-a-uuid".to_string(), "New Name".to_string());

        // When
        let result = interactor.update("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_fails_with_empty_name() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_repository = MockAuthorRepository::new();
        let interactor =
            UpdateAuthorInteractor::new(author_repository, MockTransactionManager::new());
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "".to_string());

        // When
        let result = interactor.update("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_user_id() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_repository = MockAuthorRepository::new();
        let interactor =
            UpdateAuthorInteractor::new(author_repository, MockTransactionManager::new());
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        // When
        let result = interactor.update("", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_author_success() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = DeleteAuthorInteractor::new(author_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", author_id_str).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_author_propagates_not_found() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| {
                Err(DomainError::NotFound {
                    entity_type: "author",
                    entity_id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                    user_id: "user1".to_string(),
                })
            });

        let interactor = DeleteAuthorInteractor::new(author_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn delete_author_propagates_has_associated_books() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| {
                Err(DomainError::HasAssociatedBooks {
                    author_id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                    user_id: "user1".to_string(),
                })
            });

        let interactor = DeleteAuthorInteractor::new(author_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_author_fails_with_invalid_author_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor =
            DeleteAuthorInteractor::new(author_repository, MockTransactionManager::new());

        // When
        let result = interactor.delete("user1", "not-a-uuid").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_author_fails_with_invalid_user_id() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_repository = MockAuthorRepository::new();
        let interactor =
            DeleteAuthorInteractor::new(author_repository, MockTransactionManager::new());

        // When
        let result = interactor.delete("", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}
