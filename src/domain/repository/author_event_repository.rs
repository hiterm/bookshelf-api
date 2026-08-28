use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{author::AuthorId, event::AuthorEvent, event_set::EventSetId, user::UserId},
    error::DomainError,
};

#[automock(type Transaction = ();)]
#[async_trait]
pub trait AuthorEventRepository: Send + Sync + 'static {
    type Transaction: Send;

    async fn find_by_author(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Vec<AuthorEvent>, DomainError>;

    async fn find_by_event_id(
        &self,
        user_id: &UserId,
        event_id: i64,
    ) -> Result<Option<AuthorEvent>, DomainError>;

    async fn find_by_event_set(
        &self,
        user_id: &UserId,
        event_set_id: &EventSetId,
    ) -> Result<Vec<AuthorEvent>, DomainError>;

    async fn find_by_event_set_ids(
        &self,
        user_id: &UserId,
        event_set_ids: &[EventSetId],
    ) -> Result<Vec<AuthorEvent>, DomainError>;
}
