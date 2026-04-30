use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::AuthorDto,
        book::BookDto,
        event::{AuthorEventDto, BookEventDto},
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait ListBookEventsUseCase: Send + Sync + 'static {
    async fn list(&self, user_id: &str, book_id: &str) -> Result<Vec<BookEventDto>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait ListAuthorEventsUseCase: Send + Sync + 'static {
    async fn list(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorEventDto>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait RestoreBookUseCase: Send + Sync + 'static {
    async fn restore(&self, user_id: &str, event_id: i64) -> Result<Option<BookDto>, UseCaseError>;
}

#[automock]
#[async_trait]
pub trait RestoreAuthorUseCase: Send + Sync + 'static {
    async fn restore(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<Option<AuthorDto>, UseCaseError>;
}
