use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{entity::{author::Author, user::User}, error::domain_error::DomainError};

#[async_trait]
pub trait AuthorRepository {
    async fn create(&self, user: User, author: &Author) -> Result<(), DomainError>;
    async fn find_by_id(&self, user: User, author_id: Uuid) -> Result<Option<Author>, DomainError>;
}
