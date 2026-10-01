use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use mockall::automock;
use time::OffsetDateTime;

use crate::domain::{
    entity::{
        author::{Author, AuthorId, AuthorName},
        revision::RevisionNumber,
        user::UserId,
    },
    error::DomainError,
    repository::RevisionMutationResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindOrCreateAuthorsResult {
    pub authors_by_name: HashMap<String, AuthorId>,
    pub created_author_ids: HashSet<AuthorId>,
}

/// Builds a resolved-author result in which no authors were newly created.
/// `created_author_ids` is always empty.
impl From<HashMap<String, AuthorId>> for FindOrCreateAuthorsResult {
    /// Wraps resolved author IDs without marking any authors as newly created.
    fn from(authors_by_name: HashMap<String, AuthorId>) -> Self {
        Self {
            authors_by_name,
            created_author_ids: HashSet::new(),
        }
    }
}

/// Collects resolved authors into a result in which no authors were newly
/// created. `created_author_ids` is always empty.
impl FromIterator<(String, AuthorId)> for FindOrCreateAuthorsResult {
    /// Collects resolved author IDs without marking any authors as newly created.
    fn from_iter<T: IntoIterator<Item = (String, AuthorId)>>(iter: T) -> Self {
        HashMap::from_iter(iter).into()
    }
}

#[automock(type Transaction = ();)]
#[async_trait]
pub trait AuthorRepository: Send + Sync + 'static {
    type Transaction: Send;

    /// Creates the author for the transaction owner and records its initial revision.
    /// Returns the revision number recorded by this mutation; the caller commits the transaction.
    async fn create(
        &self,
        tx: &mut Self::Transaction,
        author: &Author,
    ) -> Result<RevisionNumber, DomainError>;
    /// Returns the author belonging to the given user, or `None` when absent.
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
    /// Looks up the author for the explicit user within the supplied transaction.
    async fn find_by_id_with_tx(
        &self,
        tx: &mut Self::Transaction,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
    /// Returns all current authors belonging to the given user.
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Author>, DomainError>;
    /// Returns matching authors belonging to the given user, keyed by author ID.
    async fn find_by_ids_as_hash_map(
        &self,
        user_id: &UserId,
        author_ids: &[AuthorId],
    ) -> Result<HashMap<AuthorId, Author>, DomainError>;
    /// Resolves author names for the transaction owner, creating missing authors with the supplied timestamp.
    /// Returns the name-to-ID mapping and newly created IDs, recording initial revisions for new authors.
    async fn find_or_create_by_names(
        &self,
        tx: &mut Self::Transaction,
        names: &[AuthorName],
        created_at: OffsetDateTime,
    ) -> Result<FindOrCreateAuthorsResult, DomainError>;
    /// Updates the transaction owner's author and records a new revision.
    /// Returns the revision number recorded by this mutation; the caller commits the transaction.
    async fn update(
        &self,
        tx: &mut Self::Transaction,
        author: &Author,
    ) -> Result<RevisionNumber, DomainError>;
    /// Records the author's unchanged snapshot as a new revision in the supplied operation.
    async fn record_unchanged_revision(
        &self,
        tx: &mut Self::Transaction,
        author: &Author,
    ) -> Result<(), DomainError>;
    /// Deletes the transaction owner's author and records the operation change.
    /// The caller commits the transaction; missing or inaccessible entities return an error.
    async fn delete(
        &self,
        tx: &mut Self::Transaction,
        author_id: &AuthorId,
    ) -> Result<(), DomainError>;
    /// Restores an owned historical author snapshot as a new current revision.
    /// Returns the restored entity and newly recorded revision number, not the source number.
    /// The caller commits the transaction.
    async fn restore_revision(
        &self,
        tx: &mut Self::Transaction,
        author_id: &AuthorId,
        revision_number: i32,
    ) -> Result<RevisionMutationResult<Author>, DomainError>;
}
