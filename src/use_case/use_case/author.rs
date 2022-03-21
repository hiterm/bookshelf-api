use async_trait::async_trait;

use crate::use_case::{dto::author::Author, error::UseCaseError};

#[async_trait]
pub trait ShowAuthorUseCase {
    async fn find_by_id() -> Result<Author, UseCaseError>;
}
