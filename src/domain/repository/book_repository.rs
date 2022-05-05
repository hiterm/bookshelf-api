use async_trait::async_trait;

use crate::domain::{
    entity::{book::Book, user::UserId},
    error::DomainError,
};

#[async_trait]
pub trait BookRepository {
    async fn create(&self, user_id: &UserId, book: &Book) -> Result<(), DomainError>;
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Book>, DomainError>;
    // async fn find_by_id(&self, user: User, book_id: Uuid) -> Result<Option<Book>, DomainError>;
}
