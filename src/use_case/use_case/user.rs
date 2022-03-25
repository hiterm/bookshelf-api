use async_trait::async_trait;

use crate::use_case::{dto::user::User, error::UseCaseError};

#[async_trait]
pub trait RegisterUserUseCase: Send + Sync + 'static {
    async fn register_user(&self, user_id: &str) -> Result<User, UseCaseError>;
}
