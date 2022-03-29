use async_trait::async_trait;

use crate::{
    domain::{
        entity::user::{User as DomainUser, UserId},
        repository::user_repository::UserRepository,
    },
    use_case::{dto::user::User, error::UseCaseError, use_case::user::RegisterUserUseCase},
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
    async fn register_user(&self, user_id: &str) -> Result<User, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let user = DomainUser::new(user_id);
        self.user_repository.create(&user).await?;
        Ok(User::new(user.id.get_value()))
    }
}
