use async_trait::async_trait;

use crate::{
    domain::{
        entity::user::{User as DomainUser, UserId},
        repository::user_repository::UserRepository,
    },
    use_case::{
        dto::user::UserDto,
        error::UseCaseError,
        traits::user::{UserCommandUseCase, UserQueryUseCase},
    },
};

#[derive(Debug, Clone)]
pub struct UserQueryInteractor<UR> {
    user_repository: UR,
}

impl<UR> UserQueryInteractor<UR> {
    pub fn new(user_repository: UR) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl<UR> UserQueryUseCase for UserQueryInteractor<UR>
where
    UR: UserRepository,
{
    async fn find_by_id(&self, raw_user_id: &str) -> Result<Option<UserDto>, UseCaseError> {
        let user_id = UserId::new(raw_user_id.to_string())?;
        let user = self.user_repository.find_by_id(&user_id).await?;
        Ok(user.map(|user| UserDto::new(user.id.into_string())))
    }
}

pub struct UserCommandInteractor<UR> {
    user_repository: UR,
}

impl<UR> UserCommandInteractor<UR> {
    pub fn new(user_repository: UR) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl<UR> UserCommandUseCase for UserCommandInteractor<UR>
where
    UR: UserRepository,
{
    async fn register(&self, user_id: &str) -> Result<UserDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let user = DomainUser::new(user_id);
        self.user_repository.create(&user).await?;
        Ok(UserDto::new(user.id.into_string()))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;

    use crate::{
        domain::{
            entity::user::{User, UserId},
            repository::user_repository::MockUserRepository,
        },
        use_case::{
            error::UseCaseError,
            interactor::user::{UserCommandInteractor, UserQueryInteractor},
            traits::user::{UserCommandUseCase, UserQueryUseCase},
        },
    };

    #[tokio::test]
    async fn find_by_id_returns_user_when_found() {
        // Given
        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(always())
            .returning(|_| {
                let user_id = UserId::new("user1".to_string()).unwrap();
                Ok(Some(User::new(user_id)))
            });
        let interactor = UserQueryInteractor::new(user_repository);

        // When
        let actual = interactor.find_by_id("user1").await.unwrap();

        // Then
        assert_eq!(actual.unwrap().id, "user1");
    }

    #[tokio::test]
    async fn find_by_id_returns_none_when_not_found() {
        // Given
        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(always())
            .returning(|_| Ok(None));
        let interactor = UserQueryInteractor::new(user_repository);

        // When
        let actual = interactor.find_by_id("user1").await.unwrap();

        // Then
        assert!(actual.is_none());
    }

    #[tokio::test]
    async fn register_user_success() {
        // Given
        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_create()
            .with(always())
            .returning(|_| Ok(()));

        let interactor = UserCommandInteractor::new(user_repository);

        // When
        let result = interactor.register("user1").await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "user1");
    }

    #[tokio::test]
    async fn register_user_fails_with_empty_id() {
        // Given
        let user_repository = MockUserRepository::new();
        let interactor = UserCommandInteractor::new(user_repository);

        // When
        let result = interactor.register("").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}
