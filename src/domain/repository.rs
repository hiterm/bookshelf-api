pub mod author_repository;
pub mod book_repository;
pub mod history_repository;
pub mod transaction;
pub mod user_repository;

use crate::domain::entity::revision::RevisionNumber;

/// The entity and revision recorded by a successful repository mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionMutationResult<T> {
    pub entity: T,
    pub revision_number: RevisionNumber,
}
