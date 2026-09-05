use time::{Date, OffsetDateTime};

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::{AuthorId, AuthorName},
            book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            operation::OperationId,
            user::UserId,
        },
        error::DomainError,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevisionNumber(i32);

impl RevisionNumber {
    pub const FIRST: Self = Self(1);

    pub fn value(self) -> i32 {
        self.0
    }

    pub fn next(self) -> Result<Self, DomainError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| DomainError::Validation("revision number overflow".to_owned()))
    }
}

impl TryFrom<i32> for RevisionNumber {
    type Error = DomainError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > 0 {
            Ok(Self(value))
        } else {
            Err(DomainError::Validation(
                "revision number must be positive".to_owned(),
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookRevision {
    pub book_id: BookId,
    pub revision_number: RevisionNumber,
    pub user_id: UserId,
    pub title: BookTitle,
    pub author_ids: Vec<AuthorId>,
    pub isbn: Isbn,
    pub read: ReadFlag,
    pub owned: OwnedFlag,
    pub priority: Priority,
    pub format: BookFormat,
    pub store: BookStore,
    pub purchase_date: Option<Date>,
    pub book_created_at: OffsetDateTime,
    pub book_updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorRevision {
    pub author_id: AuthorId,
    pub revision_number: RevisionNumber,
    pub user_id: UserId,
    pub name: AuthorName,
    pub yomi: String,
    pub author_created_at: OffsetDateTime,
    pub author_updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookOperationChange {
    pub operation_id: OperationId,
    pub book_id: BookId,
    pub before_revision_number: Option<RevisionNumber>,
    pub after_revision_number: Option<RevisionNumber>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorOperationChange {
    pub operation_id: OperationId,
    pub author_id: AuthorId,
    pub before_revision_number: Option<RevisionNumber>,
    pub after_revision_number: Option<RevisionNumber>,
}

#[cfg(test)]
mod tests {
    use super::RevisionNumber;

    #[test]
    fn revision_numbers_are_positive() {
        assert!(RevisionNumber::try_from(0).is_err());
        assert!(RevisionNumber::try_from(-1).is_err());
        assert_eq!(
            RevisionNumber::try_from(1)
                .expect("one is a valid revision number")
                .value(),
            RevisionNumber::FIRST.value()
        );
    }

    #[test]
    fn next_revision_increments_by_one() {
        assert_eq!(
            RevisionNumber::try_from(41)
                .expect("positive revision")
                .next()
                .expect("revision does not overflow")
                .value(),
            42
        );
    }

    #[test]
    fn next_revision_rejects_overflow() {
        assert!(
            RevisionNumber::try_from(i32::MAX)
                .expect("positive revision")
                .next()
                .is_err()
        );
    }
}
