use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    entity::{book::Book, user::User},
    error::domain_error::DomainError,
};

#[async_trait]
pub trait BookRepository {
    async fn create(&self, user: User, book: &Book) -> Result<(), DomainError>;
    // async fn find_all(&self, user: User, book: &Book) -> Result<Vec<Book>, DomainError>;
    // async fn find_by_id(&self, user: User, book_id: Uuid) -> Result<Option<Book>, DomainError>;
}
