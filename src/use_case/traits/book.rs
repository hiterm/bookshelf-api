use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::book::{BookDto, CreateBookDto, UpdateBookDto},
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait CreateBookUseCase: Send + Sync + 'static {
    async fn create(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait UpdateBookUseCase: Send + Sync + 'static {
    async fn update(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait DeleteBookUseCase: Send + Sync + 'static {
    async fn delete(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<(), UseCaseError>;
}
