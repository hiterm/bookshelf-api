use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::{AuthorDto, CreateAuthorDto, MergeAuthorInputDto, UpdateAuthorDto},
        mutation::{
            AuthorMutationResultDto, DeleteAuthorResultDto, MutationResultDto,
            RestoreAuthorResultDto,
        },
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait AuthorQueryUseCase: Send + Sync + 'static {
    async fn find_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Option<AuthorDto>, UseCaseError>;
    async fn find_all(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError>;
    async fn find_by_ids(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, AuthorDto>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait AuthorCommandUseCase: Send + Sync + 'static {
    async fn create(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError>;
    async fn merge(
        &self,
        user_id: &str,
        input: MergeAuthorInputDto,
    ) -> Result<MutationResultDto<AuthorDto>, UseCaseError>;
    async fn update(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError>;
    async fn delete(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<DeleteAuthorResultDto, UseCaseError>;
    async fn restore(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreAuthorResultDto, UseCaseError>;
}
