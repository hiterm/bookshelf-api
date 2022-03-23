use async_trait::async_trait;

use crate::{
    domain::{entity::user::UserId, repository::user_repository::UserRepository},
    use_case::{dto::user::User, error::UseCaseError, use_case::user::LoginUseCase},
};

pub struct LoginInteractor<UR> {
    user_repository: UR,
}

impl<UR> LoginInteractor<UR> {
    pub fn new(user_repository: UR) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl<UR> LoginUseCase for LoginInteractor<UR>
where
    UR: UserRepository,
{
    async fn check_user_registration(&self, id: &str) -> Result<User, UseCaseError> {
        let user_id = UserId::new(id.to_string())?;
        let user = self.user_repository.find_by_id(&user_id).await?;

        user.ok_or(UseCaseError::NotFound {
            entity_type: "user",
            entity_id: id.to_string(),
            user_id: id.to_string(),
        })
        .map(|user| User::new(user.id.id))
    }
}
