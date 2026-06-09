use async_trait::async_trait;
use mockall::automock;
use sqlx::PgConnection;

use crate::domain::{
    entity::{
        book::{Book, BookId},
        user::UserId,
    },
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait BookRepository: Send + Sync + 'static {
    async fn create(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        book: &Book,
    ) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Option<Book>, DomainError>;
    async fn find_all(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
    ) -> Result<Vec<Book>, DomainError>;
    async fn update(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        book: &Book,
    ) -> Result<(), DomainError>;
    async fn delete(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<(), DomainError>;
    // Upserts or deletes the entity and records a restore event in one transaction.
    // book=Some means upsert; book=None means delete (only book_id is used).
    async fn restore(
        &self,
        conn: &mut PgConnection,
        user_id: &UserId,
        source_event_id: i64,
        book: Option<Book>,
    ) -> Result<(), DomainError>;
}
