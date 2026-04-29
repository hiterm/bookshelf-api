use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{book::BookId, history::BookHistory, user::UserId},
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait BookHistoryRepository: Send + Sync + 'static {
    async fn find_by_book(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Vec<BookHistory>, DomainError>;

    async fn find_by_history_id(
        &self,
        user_id: &UserId,
        history_id: i64,
    ) -> Result<Option<BookHistory>, DomainError>;
}
