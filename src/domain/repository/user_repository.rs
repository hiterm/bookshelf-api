use async_trait::async_trait;

use crate::domain::{
    entity::user::{User, UserId},
    error::DomainError,
};

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, user: &User) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;
}
