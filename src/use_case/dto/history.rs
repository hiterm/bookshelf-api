use time::OffsetDateTime;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::entity::{
        operation::{Operation, OperationDetail},
        revision::{AuthorOperationChange, AuthorRevision, BookOperationChange, BookRevision},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDto {
    pub id: String,
    pub operation_type: String,
    pub detail: Option<OperationDetail>,
    pub undo_of_operation_id: Option<String>,
    pub created_at: OffsetDateTime,
}

impl From<Operation> for OperationDto {
    fn from(value: Operation) -> Self {
        Self {
            id: value.id.to_string(),
            operation_type: value.operation_type.as_str().to_owned(),
            detail: value.detail,
            undo_of_operation_id: value.undo_of_operation_id.map(|id| id.to_string()),
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookRevisionDto {
    pub book_id: String,
    pub revision_number: i32,
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub book_created_at: OffsetDateTime,
    pub book_updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl From<BookRevision> for BookRevisionDto {
    fn from(value: BookRevision) -> Self {
        Self {
            book_id: value.book_id.to_string(),
            revision_number: value.revision_number.value(),
            title: value.title.into_string(),
            author_ids: value
                .author_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            isbn: value.isbn.into_string(),
            read: value.read.to_bool(),
            owned: value.owned.to_bool(),
            priority: value.priority.to_i32(),
            format: value.format,
            store: value.store,
            book_created_at: value.book_created_at,
            book_updated_at: value.book_updated_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorRevisionDto {
    pub author_id: String,
    pub revision_number: i32,
    pub name: String,
    pub yomi: String,
    pub author_created_at: OffsetDateTime,
    pub author_updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl From<AuthorRevision> for AuthorRevisionDto {
    fn from(value: AuthorRevision) -> Self {
        Self {
            author_id: value.author_id.to_string(),
            revision_number: value.revision_number.value(),
            name: value.name.into_string(),
            yomi: value.yomi,
            author_created_at: value.author_created_at,
            author_updated_at: value.author_updated_at,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookOperationChangeDto {
    pub operation_id: String,
    pub book_id: String,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

impl From<BookOperationChange> for BookOperationChangeDto {
    fn from(value: BookOperationChange) -> Self {
        Self {
            operation_id: value.operation_id.to_string(),
            book_id: value.book_id.to_string(),
            before_revision_number: value.before_revision_number.map(|n| n.value()),
            after_revision_number: value.after_revision_number.map(|n| n.value()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorOperationChangeDto {
    pub operation_id: String,
    pub author_id: String,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

impl From<AuthorOperationChange> for AuthorOperationChangeDto {
    fn from(value: AuthorOperationChange) -> Self {
        Self {
            operation_id: value.operation_id.to_string(),
            author_id: value.author_id.to_string(),
            before_revision_number: value.before_revision_number.map(|n| n.value()),
            after_revision_number: value.after_revision_number.map(|n| n.value()),
        }
    }
}
