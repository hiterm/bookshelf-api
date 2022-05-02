use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::{AuthorDto, CreateAuthorDto},
        user::UserDto,
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait MutationUseCase: Send + Sync + 'static {
    async fn register_user(&self, user_id: &str) -> Result<UserDto, UseCaseError>;
    async fn create_author(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorDto, UseCaseError>;
}
