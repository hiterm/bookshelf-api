use async_trait::async_trait;

use crate::use_case::{
    dto::user::User,
    error::UseCaseError,
    use_case::{mutation::MutationUseCase, user::RegisterUserUseCase},
};

pub struct MutationInteractor<RUUC> {
    register_user_use_case: RUUC,
}

impl<RUUC> MutationInteractor<RUUC> {
    pub fn new(register_user_use_case: RUUC) -> Self {
        Self {
            register_user_use_case,
        }
    }
}

#[async_trait]
impl<RUUC> MutationUseCase for MutationInteractor<RUUC>
where
    RUUC: RegisterUserUseCase,
{
    async fn register_user(&self, user_id: &str) -> Result<User, UseCaseError> {
        let user = self.register_user_use_case.register_user(user_id).await?;
        Ok(user)
    }
}
