use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        book::{CreateBookDto, ImportBookEntryDto, UpdateBookDto},
        mutation::{BookMutationResultDto, DeleteBookResultDto, ImportBooksResultDto},
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait CreateBookUseCase: Send + Sync + 'static {
    async fn create(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait UpdateBookUseCase: Send + Sync + 'static {
    async fn update(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait DeleteBookUseCase: Send + Sync + 'static {
    async fn delete(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<DeleteBookResultDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait ImportBooksUseCase: Send + Sync + 'static {
    async fn import(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<ImportBooksResultDto, UseCaseError>;
}
