use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::AuthorDto,
        book::BookDto,
        event::{AuthorEventDto, BookEventDto},
        event_set::{EventSetDetailDto, EventSetDto},
        user::UserDto,
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait QueryUseCase: Send + Sync + 'static {
    async fn find_user_by_id(&self, user_id: &str) -> Result<Option<UserDto>, UseCaseError>;
    async fn find_book_by_id(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Option<BookDto>, UseCaseError>;
    async fn find_all_books(&self, user_id: &str) -> Result<Vec<BookDto>, UseCaseError>;
    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Option<AuthorDto>, UseCaseError>;
    async fn find_all_authors(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError>;
    async fn find_author_by_ids_as_hash_map(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, AuthorDto>, UseCaseError>;
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
    async fn list_event_sets(&self, user_id: &str) -> Result<Vec<EventSetDto>, UseCaseError>;
    async fn find_event_set(
        &self,
        user_id: &str,
        event_set_id: &str,
    ) -> Result<Option<EventSetDetailDto>, UseCaseError>;
}
