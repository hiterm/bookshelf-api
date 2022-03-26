use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{
        author::{Author, AuthorId},
        user::UserId,
    },
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait AuthorRepository: Send + Sync + 'static {
    async fn create(&self, user_id: &UserId, author: &Author) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
    async fn find_all(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Author>, DomainError>;
}
