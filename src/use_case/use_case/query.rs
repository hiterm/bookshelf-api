use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{author::AuthorDto, book::BookDto, user::UserDto},
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait QueryUseCase: Send + Sync + 'static {
    async fn find_user_by_id(&self, user_id: &str) -> Result<Option<UserDto>, UseCaseError>;
    async fn find_all_books(&self, user_id: &str) -> Result<Vec<BookDto>, UseCaseError>;
    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Option<AuthorDto>, UseCaseError>;
    async fn find_all_authors(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError>;
    async fn find_author_by_ids_as_hash_map(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, AuthorDto>, UseCaseError>;
}
