use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{author::AuthorId, event::AuthorEvent, user::UserId},
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait AuthorEventRepository: Send + Sync + 'static {
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
}
