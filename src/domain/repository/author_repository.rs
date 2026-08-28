use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use mockall::automock;
use time::OffsetDateTime;

use crate::domain::{
    entity::{
        author::{Author, AuthorId, AuthorName},
        user::UserId,
    },
    error::DomainError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteAuthorEventExtra {
    Merge { destination_author_id: AuthorId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindOrCreateAuthorsResult {
    pub authors_by_name: HashMap<String, AuthorId>,
    pub created_author_ids: HashSet<AuthorId>,
}

/// Builds a resolved-author result in which no authors were newly created.
/// `created_author_ids` is always empty.
impl From<HashMap<String, AuthorId>> for FindOrCreateAuthorsResult {
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
    fn from_iter<T: IntoIterator<Item = (String, AuthorId)>>(iter: T) -> Self {
        HashMap::from_iter(iter).into()
    }
}

#[automock(type Transaction = ();)]
#[async_trait]
pub trait AuthorRepository: Send + Sync + 'static {
    type Transaction: Send;

    async fn create(&self, tx: &mut Self::Transaction, author: &Author)
    -> Result<i32, DomainError>;
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
    async fn find_by_id_with_tx(
        &self,
        tx: &mut Self::Transaction,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError>;
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Author>, DomainError>;
    async fn find_by_ids_as_hash_map(
        &self,
        user_id: &UserId,
        author_ids: &[AuthorId],
    ) -> Result<HashMap<AuthorId, Author>, DomainError>;
    // Resolves an author by name within the transaction, creating it if absent.
    // A newly inserted author records one author_event; an existing one records none.
    async fn find_or_create_by_name(
        &self,
        tx: &mut Self::Transaction,
        name: &AuthorName,
        created_at: OffsetDateTime,
    ) -> Result<AuthorId, DomainError>;
    async fn find_or_create_by_names(
        &self,
        tx: &mut Self::Transaction,
        names: &[AuthorName],
        created_at: OffsetDateTime,
    ) -> Result<FindOrCreateAuthorsResult, DomainError>;
    async fn update(&self, tx: &mut Self::Transaction, author: &Author)
    -> Result<i32, DomainError>;
    async fn record_unchanged_revision(
        &self,
        tx: &mut Self::Transaction,
        author: &Author,
    ) -> Result<(), DomainError>;
    async fn delete(
        &self,
        tx: &mut Self::Transaction,
        author_id: &AuthorId,
        extra: Option<DeleteAuthorEventExtra>,
    ) -> Result<(), DomainError>;
    // Upserts or deletes the entity and records a restore event in one transaction.
    // author=Some means upsert; author=None means delete (only author_id is used).
    async fn restore(
        &self,
        tx: &mut Self::Transaction,
        source_event_id: i64,
        author: Option<Author>,
    ) -> Result<(), DomainError>;
    async fn restore_revision(
        &self,
        tx: &mut Self::Transaction,
        author_id: &AuthorId,
        revision_number: i32,
    ) -> Result<Author, DomainError>;
}
