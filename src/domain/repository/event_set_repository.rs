use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{
        event_set::{EventSet, EventSetId},
        user::UserId,
    },
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait EventSetRepository: Send + Sync + 'static {
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<EventSet>, DomainError>;
    async fn find_by_id(
        &self,
        user_id: &UserId,
        event_set_id: &EventSetId,
    ) -> Result<Option<EventSet>, DomainError>;
}
