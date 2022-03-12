use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{entity::user::User, error::domain_error::DomainError};

#[async_trait]
pub trait UserRepository {
    async fn create(&self, user: User) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
}
