use derive_more::Display;
use getset::{Getters, Setters};
use once_cell::sync::Lazy;
use regex::Regex;
use time::OffsetDateTime;
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
}

impl std::fmt::Display for BookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id.to_hyphenated())
    }
}

impl TryFrom<&str> for BookId {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(value).map_err(|err| {
            DomainError::Validation(format!(
                r#"Failed to parse id "{}" as uuid. Message from uuid crate: {}"#,
                value, err
            ))
        })?;
        Ok(BookId { id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct BookTitle {
    #[validate(length(min = 1))]
    value: String,
}

impl_string_value_object!(BookTitle);

static ISBN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^$|^(\d-?){12}\d$)").unwrap());

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Isbn {
    // TODO: Validate check digit
    #[validate(regex = "ISBN_REGEX")]
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

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Priority {
    #[validate(range(min = 0, max = 100))]
    value: i32,
}

impl Priority {
    pub fn new(value: i32) -> Result<Priority, DomainError> {
        let priority = Self { value };
        priority.validate()?;
        Ok(priority)
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

#[derive(Debug, Clone, PartialEq, Eq, Getters, Setters)]
pub struct Book {
    #[getset(get = "pub")]
    id: BookId,
    #[getset(get = "pub", set = "pub")]
    title: BookTitle,
    #[getset(get = "pub", set = "pub")]
    author_ids: Vec<AuthorId>,
    #[getset(get = "pub", set = "pub")]
    isbn: Isbn,
    #[getset(get = "pub", set = "pub")]
    read: ReadFlag,
    #[getset(get = "pub", set = "pub")]
    owned: OwnedFlag,
    #[getset(get = "pub", set = "pub")]
    priority: Priority,
    #[getset(get = "pub", set = "pub")]
    format: BookFormat,
    #[getset(get = "pub", set = "pub")]
    store: BookStore,
    #[getset(get = "pub", set = "pub")]
    created_at: OffsetDateTime,
    #[getset(get = "pub", set = "pub")]
    updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestructureBook {
    pub id: BookId,
    pub title: BookTitle,
    pub author_ids: Vec<AuthorId>,
    pub isbn: Isbn,
    pub read: ReadFlag,
    pub owned: OwnedFlag,
    pub priority: Priority,
    pub format: BookFormat,
    pub store: BookStore,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Book {
    #[allow(clippy::too_many_arguments)]
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
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
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

    pub fn destructure(self) -> DestructureBook {
        DestructureBook {
            id: self.id,
            title: self.title,
            author_ids: self.author_ids,
            isbn: self.isbn,
            read: self.read,
            owned: self.owned,
            priority: self.priority,
            format: self.format,
            store: self.store,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::domain::entity::book::{BookFormat, BookStore};

    use super::{Isbn, Priority};

    #[test]
    fn valid_isbn_with_hyphen() {
        let isbn = Isbn::new("978-4062758574".to_owned());
        assert!(matches!(isbn, Ok(_)));
    }

    #[test]
    fn valid_isbn_without_hyphen() {
        let isbn = Isbn::new("9784062758574".to_owned());
        assert!(matches!(isbn, Ok(_)));
    }

    #[test]
    fn empty_isbn_is_valid() {
        let isbn = Isbn::new("".to_owned());
        assert!(matches!(isbn, Ok(_)));
    }

    #[test]
    fn isbn_too_short() {
        let isbn = Isbn::new("1".to_owned());
        assert!(matches!(isbn, Err(_)));
    }

    #[test]
    fn priority_0_is_valid() {
        let priority = Priority::new(0);
        assert!(matches!(priority, Ok(_)));
    }

    #[test]
    fn priority_100_is_valid() {
        let priority = Priority::new(100);
        assert!(matches!(priority, Ok(_)));
    }

    #[test]
    fn priority_negative1_is_invalid() {
        let priority = Priority::new(-1);
        assert!(matches!(priority, Err(_)));
    }

    #[test]
    fn priority_101_is_invalid() {
        let priority = Priority::new(101);
        assert!(matches!(priority, Err(_)));
    }

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
