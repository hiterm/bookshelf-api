use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::book::{BookDto, CreateBookDto},
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
