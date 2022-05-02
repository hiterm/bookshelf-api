use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{author::AuthorDto, user::UserDto},
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait QueryUseCase: Send + Sync + 'static {
    async fn find_user_by_id(&self, user_id: &str) -> Result<Option<UserDto>, UseCaseError>;
    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Option<AuthorDto>, UseCaseError>;
    async fn find_all_authors(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError>;
}
