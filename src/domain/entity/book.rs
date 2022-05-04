use derive_more::Display;
use getset::Getters;
use time::PrimitiveDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{domain::error::DomainError, impl_string_value_object};

use super::author::AuthorId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookId {
    id: Uuid,
}

impl BookId {
    pub fn new(id: Uuid) -> Result<BookId, DomainError> {
        Ok(BookId { id })
    }

    pub fn to_uuid(&self) -> Uuid {
        self.id
    }

    pub fn to_string(&self) -> String {
        self.id.to_hyphenated().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct BookTitle {
    value: String,
}

impl_string_value_object!(BookTitle);

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Isbn {
    value: String,
}

impl_string_value_object!(Isbn);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadFlag {
    value: bool,
}

impl ReadFlag {
    pub fn new(value: bool) -> ReadFlag {
        ReadFlag { value }
    }

    pub fn to_bool(&self) -> bool {
        self.value
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

    pub fn to_bool(&self) -> bool {
        self.value
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

    pub fn to_i32(&self) -> i32 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum BookFormat {
    #[display(fmt = "eBook")]
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

#[derive(Debug, Clone, PartialEq, Eq, Display)]
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

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct Book {
    #[getset(get = "pub")]
    id: BookId,
    #[getset(get = "pub")]
    title: BookTitle,
    #[getset(get = "pub")]
    author_ids: Vec<AuthorId>,
    #[getset(get = "pub")]
    isbn: Isbn,
    #[getset(get = "pub")]
    read: ReadFlag,
    #[getset(get = "pub")]
    owned: OwnedFlag,
    #[getset(get = "pub")]
    priority: Priority,
    #[getset(get = "pub")]
    format: BookFormat,
    #[getset(get = "pub")]
    store: BookStore,
    #[getset(get = "pub")]
    created_at: PrimitiveDateTime,
    #[getset(get = "pub")]
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

#[cfg(test)]
mod test {
    use crate::domain::entity::book::{BookFormat, BookStore};

    #[test]
    fn book_format_ebook_to_string() {
        assert_eq!(BookFormat::EBook.to_string(), "eBook");
    }

    #[test]
    fn book_format_printed_to_string() {
        assert_eq!(BookFormat::Printed.to_string(), "Printed");
    }

    #[test]
    fn book_format_unknown_to_string() {
        assert_eq!(BookFormat::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn book_store_kindle_to_string() {
        assert_eq!(BookStore::Kindle.to_string(), "Kindle");
    }

    #[test]
    fn book_store_unknown_to_string() {
        assert_eq!(BookStore::Unknown.to_string(), "Unknown");
    }
}
