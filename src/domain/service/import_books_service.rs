use async_trait::async_trait;
use mockall::automock;
use time::OffsetDateTime;

use crate::common::types::{BookFormat, BookStore};
use crate::domain::entity::{
    author::AuthorName,
    book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
    user::UserId,
};
use crate::domain::error::DomainError;

#[derive(Clone)]
pub struct ImportBookInput {
    pub book_id: BookId,
    pub title: BookTitle,
    pub author_names: Vec<AuthorName>,
    pub isbn: Isbn,
    pub read: ReadFlag,
    pub owned: OwnedFlag,
    pub priority: Priority,
    pub format: BookFormat,
    pub store: BookStore,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[automock]
#[async_trait]
pub trait ImportBooksService: Send + Sync + 'static {
    async fn import(
        &self,
        user_id: &UserId,
        books: Vec<ImportBookInput>,
    ) -> Result<Vec<Book>, DomainError>;
}
