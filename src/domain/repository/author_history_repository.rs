use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{
        author::AuthorId,
        history::AuthorHistory,
        user::UserId,
    },
    error::DomainError,
};

#[automock]
#[async_trait]
pub trait AuthorHistoryRepository: Send + Sync + 'static {
    async fn find_by_author(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Vec<AuthorHistory>, DomainError>;

    async fn find_by_history_id(
        &self,
        user_id: &UserId,
        history_id: i64,
    ) -> Result<Option<AuthorHistory>, DomainError>;
}
