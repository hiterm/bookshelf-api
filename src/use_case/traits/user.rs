use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{dto::user::UserDto, error::UseCaseError};

#[automock]
#[async_trait]
pub trait RegisterUserUseCase: Send + Sync + 'static {
    async fn register_user(&self, user_id: &str) -> Result<UserDto, UseCaseError>;
}
