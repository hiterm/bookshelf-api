use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName, AuthorUpdate},
            event::EventSetOperation,
            user::UserId,
        },
        repository::{author_repository::AuthorRepository, transaction::TransactionManager},
    },
    use_case::{
        dto::author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto},
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
    ) -> Result<AuthorDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let uuid = Uuid::new_v4();
        let author_id = AuthorId::new(uuid);
        let author_name = AuthorName::new(author_data.name)?;
        let author = Author::new(author_id, author_name)?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::CreateAuthor)
            .await?;
        self.author_repository
            .create(&mut tx, &user_id, &author)
            .await?;
        self.transaction_manager.commit(tx).await?;

        Ok(author.into())
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
    ) -> Result<AuthorDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_data.id.as_str())?;
        let author_name = AuthorName::new(author_data.name)?;

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

        author.update(AuthorUpdate { name: author_name });

        self.author_repository
            .update(&mut tx, &user_id, &author)
            .await?;
        self.transaction_manager.commit(tx).await?;

        Ok(author.into())
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
    async fn delete(&self, user_id: &str, author_id: &str) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::DeleteAuthor)
            .await?;
        self.author_repository
            .delete(&mut tx, &user_id, &author_id)
            .await?;
        self.transaction_manager.commit(tx).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;

    use crate::{
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
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = CreateAuthorInteractor::new(author_repository, make_transaction_manager());
        let author_data = CreateAuthorDto::new("Test Author".to_string());

        // When
        let result = interactor.create("user1", author_data).await;

        // Then
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.name, "Test Author");
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
    async fn update_author_success() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let existing_author = Author::new(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Old Name".to_string()).unwrap(),
        )
        .unwrap();

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .with(always(), always(), always())
            .returning(move |_, _, _| Ok(Some(existing_author.clone())));
        author_repository
            .expect_update()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = UpdateAuthorInteractor::new(author_repository, make_transaction_manager());
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        // When
        let result = interactor.update("user1", author_data).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "New Name");
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
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

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
            .with(always(), always(), always())
            .returning(|_, _, _| {
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
            .with(always(), always(), always())
            .returning(|_, _, _| {
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
