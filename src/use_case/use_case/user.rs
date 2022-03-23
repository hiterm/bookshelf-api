use async_trait::async_trait;

use crate::use_case::{dto::user::User, error::UseCaseError};

#[async_trait]
pub trait LoginUseCase: Send + Sync + 'static {
    async fn check_user_registration(&self, id: &str) -> Result<User, UseCaseError>;
}
