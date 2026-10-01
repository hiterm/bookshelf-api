use std::collections::HashMap;

use async_trait::async_trait;
use mockall::automock;

use crate::domain::{
    entity::{
        author::AuthorId,
        book::{Book, BookId},
        revision::RevisionNumber,
        user::UserId,
    },
    error::DomainError,
    repository::RevisionMutationResult,
};

#[automock(type Transaction = ();)]
#[async_trait]
pub trait BookRepository: Send + Sync + 'static {
    type Transaction: Send;

    /// Creates the book for the transaction owner and records its initial revision.
    /// Returns the revision number recorded by this mutation; the caller commits the transaction.
    async fn create(
        &self,
        tx: &mut Self::Transaction,
        book: &Book,
    ) -> Result<RevisionNumber, DomainError>;
    /// Creates the books, author links, initial revisions, and operation changes in the supplied transaction.
    async fn create_all(
        &self,
        tx: &mut Self::Transaction,
        books: &[Book],
    ) -> Result<(), DomainError>;
    /// Returns the book belonging to the given user, or `None` when absent.
    async fn find_by_id(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Option<Book>, DomainError>;
    /// Looks up the book for the explicit user within the supplied transaction.
    async fn find_by_id_with_tx(
        &self,
        tx: &mut Self::Transaction,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Option<Book>, DomainError>;
    /// Returns all current books belonging to the given user.
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Book>, DomainError>;
    /// Groups the given user's books by requested author ID.
    /// Every requested ID remains in the map, with an empty list when no books match.
    async fn find_by_author_ids_as_hash_map(
        &self,
        user_id: &UserId,
        author_ids: &[AuthorId],
    ) -> Result<HashMap<AuthorId, Vec<Book>>, DomainError>;
    /// Locks the given user's matching books in ID order and reads their current author links.
    async fn find_by_author_id_with_tx(
        &self,
        tx: &mut Self::Transaction,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Vec<Book>, DomainError>;
    /// Updates the transaction owner's book and records a new revision.
    /// Returns the revision number recorded by this mutation; the caller commits the transaction.
    async fn update(
        &self,
        tx: &mut Self::Transaction,
        book: &Book,
    ) -> Result<RevisionNumber, DomainError>;
    /// Updates owned books and author links and records their revisions in the supplied transaction.
    async fn update_all(
        &self,
        tx: &mut Self::Transaction,
        books: &[Book],
    ) -> Result<(), DomainError>;
    /// Deletes the transaction owner's book and records the operation change.
    /// The caller commits the transaction; missing or inaccessible entities return an error.
    async fn delete(&self, tx: &mut Self::Transaction, book_id: &BookId)
    -> Result<(), DomainError>;
    /// Restores an owned historical book snapshot as a new current revision.
    /// Returns the restored entity and newly recorded revision number, not the source number.
    /// The caller commits the transaction.
    async fn restore_revision(
        &self,
        tx: &mut Self::Transaction,
        book_id: &BookId,
        revision_number: i32,
    ) -> Result<RevisionMutationResult<Book>, DomainError>;
}
