use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName},
            user::UserId,
        },
        repository::author_repository::AuthorRepository,
    },
    use_case::{
        dto::author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto},
        error::UseCaseError,
        traits::author::{CreateAuthorUseCase, DeleteAuthorUseCase, UpdateAuthorUseCase},
    },
};

pub struct CreateAuthorInteractor<AR> {
    author_repository: AR,
    pool: PgPool,
}

impl<AR> CreateAuthorInteractor<AR> {
    pub fn new(author_repository: AR, pool: PgPool) -> Self {
        Self {
            author_repository,
            pool,
        }
    }
}

#[async_trait]
impl<AR> CreateAuthorUseCase for CreateAuthorInteractor<AR>
where
    AR: AuthorRepository,
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

        let mut tx = self.pool.begin().await?;
        self.author_repository
            .create(&mut tx, &user_id, &author)
            .await?;
        tx.commit().await?;

        Ok(author.into())
    }
}

pub struct UpdateAuthorInteractor<AR> {
    author_repository: AR,
    pool: PgPool,
}

impl<AR> UpdateAuthorInteractor<AR> {
    pub fn new(author_repository: AR, pool: PgPool) -> Self {
        Self {
            author_repository,
            pool,
        }
    }
}

#[async_trait]
impl<AR> UpdateAuthorUseCase for UpdateAuthorInteractor<AR>
where
    AR: AuthorRepository,
{
    async fn update(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_data.id.as_str())?;
        let author_name = AuthorName::new(author_data.name)?;
        let author = Author::new(author_id, author_name)?;

        let mut tx = self.pool.begin().await?;
        self.author_repository
            .update(&mut tx, &user_id, &author)
            .await?;
        tx.commit().await?;

        Ok(author.into())
    }
}

pub struct DeleteAuthorInteractor<AR> {
    author_repository: AR,
    pool: PgPool,
}

impl<AR> DeleteAuthorInteractor<AR> {
    pub fn new(author_repository: AR, pool: PgPool) -> Self {
        Self {
            author_repository,
            pool,
        }
    }
}

#[async_trait]
impl<AR> DeleteAuthorUseCase for DeleteAuthorInteractor<AR>
where
    AR: AuthorRepository,
{
    async fn delete(&self, user_id: &str, author_id: &str) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;

        let mut tx = self.pool.begin().await?;
        self.author_repository
            .delete(&mut tx, &user_id, &author_id)
            .await?;
        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;

    use crate::{
        domain::{error::DomainError, repository::author_repository::MockAuthorRepository},
        use_case::{
            dto::author::{CreateAuthorDto, UpdateAuthorDto},
            error::UseCaseError,
            interactor::author::{
                CreateAuthorInteractor, DeleteAuthorInteractor, UpdateAuthorInteractor,
            },
            traits::author::{CreateAuthorUseCase, DeleteAuthorUseCase, UpdateAuthorUseCase},
        },
    };

    fn dummy_pool() -> sqlx::PgPool {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".to_string());
        sqlx::PgPool::connect_lazy(&url).unwrap()
    }

    #[tokio::test]
    async fn create_author_success() {
        // Given
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_create()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = CreateAuthorInteractor::new(author_repository, dummy_pool());
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
        let interactor = CreateAuthorInteractor::new(author_repository, dummy_pool());
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
        let interactor = CreateAuthorInteractor::new(author_repository, dummy_pool());
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

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_update()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = UpdateAuthorInteractor::new(author_repository, dummy_pool());
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
            .expect_update()
            .with(always(), always(), always())
            .returning(|_, _, _| {
                Err(DomainError::NotFound {
                    entity_type: "author",
                    entity_id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                    user_id: "user1".to_string(),
                })
            });

        let interactor = UpdateAuthorInteractor::new(author_repository, dummy_pool());
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
        let interactor = UpdateAuthorInteractor::new(author_repository, dummy_pool());
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
        let interactor = UpdateAuthorInteractor::new(author_repository, dummy_pool());
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
        let interactor = UpdateAuthorInteractor::new(author_repository, dummy_pool());
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

        let interactor = DeleteAuthorInteractor::new(author_repository, dummy_pool());

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

        let interactor = DeleteAuthorInteractor::new(author_repository, dummy_pool());

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

        let interactor = DeleteAuthorInteractor::new(author_repository, dummy_pool());

        // When
        let result = interactor.delete("user1", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_author_fails_with_invalid_author_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = DeleteAuthorInteractor::new(author_repository, dummy_pool());

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
        let interactor = DeleteAuthorInteractor::new(author_repository, dummy_pool());

        // When
        let result = interactor.delete("", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}
