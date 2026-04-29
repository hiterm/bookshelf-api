use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto},
        book::{BookDto, CreateBookDto, UpdateBookDto},
        user::UserDto,
    },
    error::UseCaseError,
};

#[automock]
#[async_trait]
pub trait MutationUseCase: Send + Sync + 'static {
    async fn register_user(&self, user_id: &str) -> Result<UserDto, UseCaseError>;
    async fn create_book(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookDto, UseCaseError>;
    async fn update_book(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookDto, UseCaseError>;
    async fn delete_book(&self, user_id: &str, book_id: &str) -> Result<(), UseCaseError>;
    async fn create_author(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorDto, UseCaseError>;
    async fn update_author(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorDto, UseCaseError>;
    async fn delete_author(&self, user_id: &str, author_id: &str) -> Result<(), UseCaseError>;
    async fn restore_book(
        &self,
        user_id: &str,
        history_id: i64,
    ) -> Result<BookDto, UseCaseError>;
    async fn restore_author(
        &self,
        user_id: &str,
        history_id: i64,
    ) -> Result<AuthorDto, UseCaseError>;
}
