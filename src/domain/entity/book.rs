// TODO: 消す
#![allow(warnings, unused)]

use getset::Getters;
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::domain::error::DomainError;

use super::author::Author;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
struct BookId {
    #[getset(get = "pub")]
    id: Uuid,
}

impl BookId {
    pub fn new(id: Uuid) -> Result<BookId, DomainError> {
        Ok(BookId { id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
struct Isbn {
    #[getset(get = "pub")]
    value: String,
}

impl Isbn {
    pub fn new(value: String) -> Result<Isbn, DomainError> {
        Ok(Isbn { value })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
struct ReadFlag {
    #[getset(get = "pub")]
    value: bool,
}

impl ReadFlag {
    pub fn new(value: bool) -> Result<ReadFlag, DomainError> {
        Ok(ReadFlag { value })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
struct OwnedFlag {
    #[getset(get = "pub")]
    value: bool,
}

impl OwnedFlag {
    pub fn new(value: bool) -> Result<OwnedFlag, DomainError> {
        Ok(OwnedFlag { value })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
struct Priority {
    #[getset(get = "pub")]
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
}

pub enum BookStore {
    Kindle,
}

// TODO: title
pub struct Book {
    id: BookId,
    authors: Vec<Author>,
    isbn: Option<Isbn>,
    read: ReadFlag,
    owned: OwnedFlag,
    priority: Priority,
    format: Option<BookFormat>,
    store: Option<BookStore>,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}
