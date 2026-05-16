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

/// NOTE: This is a temporary repository introduced because existing
/// PgBookRepository and PgAuthorRepository do not accept external
/// transactions, preventing use-case layer from orchestrating multiple
/// repositories within a single transaction.
/// When Unit of Work pattern is introduced, this trait should be removed
/// and its responsibilities merged back into the aggregate repositories.
#[automock]
#[async_trait]
pub trait ImportBooksRepository: Send + Sync + 'static {
    async fn import(
        &self,
        user_id: &UserId,
        books: Vec<ImportBookInput>,
    ) -> Result<Vec<Book>, DomainError>;
}
