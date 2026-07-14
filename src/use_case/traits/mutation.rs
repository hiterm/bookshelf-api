use async_trait::async_trait;
use mockall::automock;

use crate::use_case::{
    dto::{
        author::{CreateAuthorDto, UpdateAuthorDto},
        book::{CreateBookDto, ImportBookEntryDto, UpdateBookDto},
        mutation::{
            AuthorMutationResultDto, BookMutationResultDto, DeleteAuthorResultDto,
            DeleteBookResultDto, ImportBooksResultDto, RestoreAuthorResultDto,
            RestoreBookResultDto,
        },
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
    ) -> Result<BookMutationResultDto, UseCaseError>;
    async fn update_book(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError>;
    async fn delete_book(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<DeleteBookResultDto, UseCaseError>;
    async fn create_author(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError>;
    async fn update_author(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError>;
    async fn delete_author(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<DeleteAuthorResultDto, UseCaseError>;
    async fn restore_book(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreBookResultDto, UseCaseError>;
    async fn restore_author(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreAuthorResultDto, UseCaseError>;
    async fn import_books(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<ImportBooksResultDto, UseCaseError>;
}
