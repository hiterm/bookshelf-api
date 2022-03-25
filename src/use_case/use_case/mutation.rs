use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{dto::user::User, error::UseCaseError};

#[automock]
#[async_trait]
pub trait MutationUseCase: Send + Sync + 'static {
    async fn register_user(&self, user_id: &str) -> Result<User, UseCaseError>;
}
