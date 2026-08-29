use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::history::{
        AuthorOperationChangeDto, AuthorRevisionDto, BookOperationChangeDto, BookRevisionDto,
        OperationDto,
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait HistoryQueryUseCase: Send + Sync + 'static {
    async fn operations(&self, user_id: &str) -> Result<Vec<OperationDto>, UseCaseError>;
    async fn operation(
        &self,
        user_id: &str,
        operation_id: &str,
    ) -> Result<Option<OperationDto>, UseCaseError>;
    async fn is_operation_undoable(
        &self,
        user_id: &str,
        operation_id: &str,
    ) -> Result<bool, UseCaseError>;
    async fn book_revisions(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Vec<BookRevisionDto>, UseCaseError>;
    async fn book_revision(
        &self,
        user_id: &str,
        book_id: &str,
        revision_number: i32,
    ) -> Result<Option<BookRevisionDto>, UseCaseError>;
    async fn author_revisions(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorRevisionDto>, UseCaseError>;
    async fn author_revision(
        &self,
        user_id: &str,
        author_id: &str,
        revision_number: i32,
    ) -> Result<Option<AuthorRevisionDto>, UseCaseError>;
    async fn book_changes(
        &self,
        user_id: &str,
        operation_ids: &[String],
    ) -> Result<HashMap<String, Vec<BookOperationChangeDto>>, UseCaseError>;
    async fn author_changes(
        &self,
        user_id: &str,
        operation_ids: &[String],
    ) -> Result<HashMap<String, Vec<AuthorOperationChangeDto>>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait HistoryCommandUseCase: Send + Sync + 'static {
    async fn undo_operation(
        &self,
        user_id: &str,
        operation_id: &str,
    ) -> Result<String, UseCaseError>;
}
