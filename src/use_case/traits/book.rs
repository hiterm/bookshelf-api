use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        book::{BookDto, CreateBookDto, ImportBookEntryDto, UpdateBookDto},
        mutation::{
            BookMutationResultDto, DeleteBookResultDto, ImportBooksResultDto, RestoreBookResultDto,
        },
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait BookQueryUseCase: Send + Sync + 'static {
    async fn find_by_id(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Option<BookDto>, UseCaseError>;
    async fn find_all(&self, user_id: &str) -> Result<Vec<BookDto>, UseCaseError>;
    async fn find_by_author_ids(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, Vec<BookDto>>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait BookCommandUseCase: Send + Sync + 'static {
    async fn create(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError>;
    async fn update(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError>;
    async fn delete(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<DeleteBookResultDto, UseCaseError>;
    async fn import(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<ImportBooksResultDto, UseCaseError>;
    async fn restore(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreBookResultDto, UseCaseError>;
}
