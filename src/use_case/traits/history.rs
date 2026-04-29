use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::AuthorDto,
        book::BookDto,
        history::{AuthorHistoryDto, BookHistoryDto},
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait ListBookHistoryUseCase: Send + Sync + 'static {
    async fn list(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Vec<BookHistoryDto>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait ListAuthorHistoryUseCase: Send + Sync + 'static {
    async fn list(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorHistoryDto>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait RestoreBookUseCase: Send + Sync + 'static {
    async fn restore(
        &self,
        user_id: &str,
        history_id: i64,
    ) -> Result<BookDto, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait RestoreAuthorUseCase: Send + Sync + 'static {
    async fn restore(
        &self,
        user_id: &str,
        history_id: i64,
    ) -> Result<AuthorDto, UseCaseError>;
}
