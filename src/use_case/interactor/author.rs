use async_trait::async_trait;
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
        dto::author::{AuthorDto, CreateAuthorDto},
        error::UseCaseError,
        traits::author::CreateAuthorUseCase,
    },
};

pub struct CreateAuthorInteractor<AR> {
    author_repository: AR,
}

impl<AR> CreateAuthorInteractor<AR> {
    pub fn new(author_repository: AR) -> Self {
        Self { author_repository }
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
        self.author_repository.create(&user_id, &author).await?;

        Ok(author.into())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;

    use crate::{
        domain::repository::author_repository::MockAuthorRepository,
        use_case::{
            dto::author::CreateAuthorDto, error::UseCaseError,
            interactor::author::CreateAuthorInteractor, traits::author::CreateAuthorUseCase,
        },
    };

    #[tokio::test]
    async fn create_author_success() {
        // Given
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_create()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = CreateAuthorInteractor::new(author_repository);
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
        let interactor = CreateAuthorInteractor::new(author_repository);
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
        let interactor = CreateAuthorInteractor::new(author_repository);
        let author_data = CreateAuthorDto::new("Test Author".to_string());

        // When
        let result = interactor.create("", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}
