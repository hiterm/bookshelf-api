use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{
        author::AuthorId,
        book::BookId,
        operation::{Operation, OperationId},
        revision::{
            AuthorOperationChange, AuthorRevision, BookOperationChange, BookRevision,
            RevisionNumber,
        },
        user::UserId,
    },
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait HistoryRepository: Send + Sync + 'static {
    async fn find_operations(&self, user_id: &UserId) -> Result<Vec<Operation>, DomainError>;
    async fn find_operation(
        &self,
        user_id: &UserId,
        operation_id: &OperationId,
    ) -> Result<Option<Operation>, DomainError>;
    async fn find_book_revisions(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Vec<BookRevision>, DomainError>;
    async fn find_book_revision(
        &self,
        user_id: &UserId,
        book_id: &BookId,
        revision_number: RevisionNumber,
    ) -> Result<Option<BookRevision>, DomainError>;
    async fn find_author_revisions(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Vec<AuthorRevision>, DomainError>;
    async fn find_author_revision(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
        revision_number: RevisionNumber,
    ) -> Result<Option<AuthorRevision>, DomainError>;
    async fn find_book_changes_by_operation_ids(
        &self,
        user_id: &UserId,
        operation_ids: &[OperationId],
    ) -> Result<HashMap<OperationId, Vec<BookOperationChange>>, DomainError>;
    async fn find_author_changes_by_operation_ids(
        &self,
        user_id: &UserId,
        operation_ids: &[OperationId],
    ) -> Result<HashMap<OperationId, Vec<AuthorOperationChange>>, DomainError>;
}
