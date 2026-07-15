use async_trait::async_trait;

use crate::{
    domain::{
        entity::user::{User as DomainUser, UserId},
        repository::user_repository::UserRepository,
    },
    use_case::{dto::user::UserDto, error::UseCaseError, traits::user::RegisterUserUseCase},
};

pub struct RegisterUserInteractor<UR> {
    user_repository: UR,
}

impl<UR> RegisterUserInteractor<UR> {
    pub fn new(user_repository: UR) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl<UR> RegisterUserUseCase for RegisterUserInteractor<UR>
where
    UR: UserRepository,
{
    async fn register_user(&self, user_id: &str) -> Result<UserDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        if self.user_repository.find_by_id(&user_id).await?.is_some() {
            return Err(UseCaseError::Conflict(format!(
                r#"user "{}" already exists"#,
                user_id.as_str()
            )));
        }
        let user = DomainUser::new(user_id);
        self.user_repository.create(&user).await?;
        Ok(UserDto::new(user.id.into_string()))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;

    use crate::{
        domain::repository::user_repository::MockUserRepository,
        use_case::{
            error::UseCaseError, interactor::user::RegisterUserInteractor,
            traits::user::RegisterUserUseCase,
        },
    };

    #[tokio::test]
    async fn register_user_success() {
        // Given
        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(always())
            .returning(|_| Ok(None));
        user_repository
            .expect_create()
            .with(always())
            .returning(|_| Ok(()));

        let interactor = RegisterUserInteractor::new(user_repository);

        // When
        let result = interactor.register_user("user1").await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "user1");
    }

    #[tokio::test]
    async fn register_user_fails_with_empty_id() {
        // Given
        let user_repository = MockUserRepository::new();
        let interactor = RegisterUserInteractor::new(user_repository);

        // When
        let result = interactor.register_user("").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn register_user_fails_when_user_already_exists() {
        // Given
        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_find_by_id()
            .with(always())
            .returning(|id| Ok(Some(crate::domain::entity::user::User::new(id.clone()))));

        let interactor = RegisterUserInteractor::new(user_repository);

        // When
        let result = interactor.register_user("user1").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Conflict(_))));
    }
}
