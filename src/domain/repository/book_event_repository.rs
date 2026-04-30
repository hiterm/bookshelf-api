use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{book::BookId, history::BookEvent, user::UserId},
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait BookEventRepository: Send + Sync + 'static {
    async fn find_by_book(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Vec<BookEvent>, DomainError>;

    async fn find_by_event_id(
        &self,
        user_id: &UserId,
        event_id: i64,
    ) -> Result<Option<BookEvent>, DomainError>;
}
