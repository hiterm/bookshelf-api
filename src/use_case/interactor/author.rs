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
        dto::author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto},
        error::UseCaseError,
        traits::author::{CreateAuthorUseCase, DeleteAuthorUseCase, UpdateAuthorUseCase},
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

pub struct UpdateAuthorInteractor<AR> {
    author_repository: AR,
}

impl<AR> UpdateAuthorInteractor<AR> {
    pub fn new(author_repository: AR) -> Self {
        Self { author_repository }
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
        let existing = self
            .author_repository
            .find_by_id(&user_id, &author_id)
            .await?;
        if existing.is_none() {
            return Err(UseCaseError::NotFound {
                entity_type: "author",
                entity_id: author_data.id,
                user_id: user_id.into_string(),
            });
        }
        let author_name = AuthorName::new(author_data.name)?;
        let author = Author::new(author_id, author_name)?;
        self.author_repository.update(&user_id, &author).await?;
        Ok(author.into())
    }
}

pub struct DeleteAuthorInteractor<AR> {
    author_repository: AR,
}

impl<AR> DeleteAuthorInteractor<AR> {
    pub fn new(author_repository: AR) -> Self {
        Self { author_repository }
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
        self.author_repository.delete(&user_id, &author_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;

    use crate::{
        domain::{
            entity::author::{Author, AuthorId, AuthorName},
            repository::author_repository::MockAuthorRepository,
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

    const VALID_AUTHOR_ID: &str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
    const VALID_USER_ID: &str = "user1";

    fn make_author(id_str: &str, name: &str) -> Author {
        Author::new(
            AuthorId::try_from(id_str).unwrap(),
            AuthorName::new(name.to_string()).unwrap(),
        )
        .unwrap()
    }

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

    // --- UpdateAuthorInteractor tests ---

    #[tokio::test]
    async fn update_author_success() {
        // Given
        let existing_author = make_author(VALID_AUTHOR_ID, "Old Name");

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id()
            .with(always(), always())
            .returning(move |_, _| Ok(Some(make_author(VALID_AUTHOR_ID, "Old Name"))));
        author_repository
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = UpdateAuthorInteractor::new(author_repository);
        let author_data =
            UpdateAuthorDto::new(VALID_AUTHOR_ID.to_string(), "Updated Name".to_string());

        // When
        let result = interactor.update(VALID_USER_ID, author_data).await;

        // Then
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.id, existing_author.id().to_string());
        assert_eq!(dto.name, "Updated Name");
    }

    #[tokio::test]
    async fn update_author_not_found() {
        // Given
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id()
            .with(always(), always())
            .returning(|_, _| Ok(None));

        let interactor = UpdateAuthorInteractor::new(author_repository);
        let author_data =
            UpdateAuthorDto::new(VALID_AUTHOR_ID.to_string(), "Updated Name".to_string());

        // When
        let result = interactor.update(VALID_USER_ID, author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn update_author_fails_with_empty_name() {
        // Given
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id()
            .with(always(), always())
            .returning(|_, _| Ok(Some(make_author(VALID_AUTHOR_ID, "Old Name"))));

        let interactor = UpdateAuthorInteractor::new(author_repository);
        let author_data = UpdateAuthorDto::new(VALID_AUTHOR_ID.to_string(), "".to_string());

        // When
        let result = interactor.update(VALID_USER_ID, author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_user_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = UpdateAuthorInteractor::new(author_repository);
        let author_data =
            UpdateAuthorDto::new(VALID_AUTHOR_ID.to_string(), "Updated Name".to_string());

        // When
        let result = interactor.update("", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_author_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = UpdateAuthorInteractor::new(author_repository);
        let author_data =
            UpdateAuthorDto::new("not-a-valid-uuid".to_string(), "Updated Name".to_string());

        // When
        let result = interactor.update(VALID_USER_ID, author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    // --- DeleteAuthorInteractor tests ---

    #[tokio::test]
    async fn delete_author_success() {
        // Given
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = DeleteAuthorInteractor::new(author_repository);

        // When
        let result = interactor.delete(VALID_USER_ID, VALID_AUTHOR_ID).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_author_fails_with_invalid_user_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = DeleteAuthorInteractor::new(author_repository);

        // When
        let result = interactor.delete("", VALID_AUTHOR_ID).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_author_fails_with_invalid_author_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = DeleteAuthorInteractor::new(author_repository);

        // When
        let result = interactor.delete(VALID_USER_ID, "not-a-valid-uuid").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}