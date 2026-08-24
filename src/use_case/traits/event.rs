use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        event::{AuthorEventDto, BookEventDto},
        event_set::EventSetDto,
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait EventQueryUseCase: Send + Sync + 'static {
    async fn list_book_events(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Vec<BookEventDto>, UseCaseError>;
    async fn list_author_events(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorEventDto>, UseCaseError>;
    async fn list_book_events_by_event_set_ids(
        &self,
        user_id: &str,
        event_set_ids: &[String],
    ) -> Result<Vec<BookEventDto>, UseCaseError>;
    async fn list_author_events_by_event_set_ids(
        &self,
        user_id: &str,
        event_set_ids: &[String],
    ) -> Result<Vec<AuthorEventDto>, UseCaseError>;
    async fn list_event_sets(&self, user_id: &str) -> Result<Vec<EventSetDto>, UseCaseError>;
    async fn find_event_set(
        &self,
        user_id: &str,
        event_set_id: &str,
    ) -> Result<Option<EventSetDto>, UseCaseError>;
}
