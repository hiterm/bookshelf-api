use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::domain::error::DomainError;

use super::author::AuthorId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookId {
    id: Uuid,
}

impl BookId {
    pub fn new(id: Uuid) -> Result<BookId, DomainError> {
        Ok(BookId { id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookTitle {
    value: String,
}

impl BookTitle {
    pub fn new(name: String) -> Result<BookTitle, DomainError> {
        Ok(BookTitle { value: name })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Isbn {
    value: String,
}

impl Isbn {
    pub fn new(value: String) -> Result<Isbn, DomainError> {
        Ok(Isbn { value })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadFlag {
    value: bool,
}

impl ReadFlag {
    pub fn new(value: bool) -> ReadFlag {
        ReadFlag { value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedFlag {
    value: bool,
}

impl OwnedFlag {
    pub fn new(value: bool) -> OwnedFlag {
        OwnedFlag { value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Priority {
    value: i32,
}

impl Priority {
    pub fn new(value: i32) -> Result<Priority, DomainError> {
        Ok(Priority { value })
    }
}

pub enum BookFormat {
    EBook,
    Printed,
    Unknown,
}

impl TryFrom<&str> for BookFormat {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "eBook" => Ok(BookFormat::EBook),
            "Printed" => Ok(BookFormat::Printed),
            "Unknown" => Ok(BookFormat::Unknown),
            _ => Err(DomainError::Validation(format!(
                "{} is not valid format",
                value
            ))),
        }
    }
}

pub enum BookStore {
    Kindle,
    Unknown,
}

impl TryFrom<&str> for BookStore {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Kindle" => Ok(BookStore::Kindle),
            "Unknown" => Ok(BookStore::Unknown),
            _ => Err(DomainError::Validation(format!(
                "{} is not valid store",
                value
            ))),
        }
    }
}

pub struct Book {
    id: BookId,
    title: BookTitle,
    author_ids: Vec<AuthorId>,
    isbn: Isbn,
    read: ReadFlag,
    owned: OwnedFlag,
    priority: Priority,
    format: BookFormat,
    store: BookStore,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

impl Book {
    pub fn new(
        id: BookId,
        title: BookTitle,
        author_ids: Vec<AuthorId>,
        isbn: Isbn,
        read: ReadFlag,
        owned: OwnedFlag,
        priority: Priority,
        format: BookFormat,
        store: BookStore,
        created_at: PrimitiveDateTime,
        updated_at: PrimitiveDateTime,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            created_at,
            updated_at,
        })
    }
}
