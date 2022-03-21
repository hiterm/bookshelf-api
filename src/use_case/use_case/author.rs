use async_trait::async_trait;

use crate::use_case::{dto::author::Author, error::UseCaseError};

#[async_trait]
pub trait ShowAuthorUseCase: Send + Sync + 'static {
    async fn find_by_id(&self, user_id: &str, author_id: &str) -> Result<Author, UseCaseError>;
}
