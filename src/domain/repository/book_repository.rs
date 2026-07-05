use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{
        book::{Book, BookId},
        user::UserId,
    },
    error::DomainError,
};

#[automock(type Transaction = ();)]
#[async_trait]
pub trait BookRepository: Send + Sync + 'static {
    type Transaction: Send;

    async fn create(&self, tx: &mut Self::Transaction, book: &Book) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Option<Book>, DomainError>;
    async fn find_by_id_with_tx(
        &self,
        tx: &mut Self::Transaction,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Option<Book>, DomainError>;
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Book>, DomainError>;
    async fn update(&self, tx: &mut Self::Transaction, book: &Book) -> Result<(), DomainError>;
    async fn delete(&self, tx: &mut Self::Transaction, book_id: &BookId)
    -> Result<(), DomainError>;
    // Upserts or deletes the entity and records a restore event in one transaction.
    // book=Some means upsert; book=None means delete (only book_id is used).
    async fn restore(
        &self,
        tx: &mut Self::Transaction,
        source_event_id: i64,
        book: Option<Book>,
    ) -> Result<(), DomainError>;
}
