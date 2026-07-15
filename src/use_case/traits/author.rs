use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::{CreateAuthorDto, UpdateAuthorDto},
        mutation::{AuthorMutationResultDto, DeleteAuthorResultDto},
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait CreateAuthorUseCase: Send + Sync + 'static {
    async fn create(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait UpdateAuthorUseCase: Send + Sync + 'static {
    async fn update(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait DeleteAuthorUseCase: Send + Sync + 'static {
    async fn delete(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<DeleteAuthorResultDto, UseCaseError>;
}
