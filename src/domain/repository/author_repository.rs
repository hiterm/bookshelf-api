use async_trait::async_trait;

use crate::domain::{
    entity::{
        author::{Author, AuthorId},
        user::UserId,
    },
    error::DomainError,
};

#[async_trait]
pub trait AuthorRepository: Send + Sync {
    async fn create(&self, user_id: &UserId, author: &Author) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
}
