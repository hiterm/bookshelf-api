use getset::Getters;
use regex::Regex;
use std::sync::LazyLock;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    common::{
        time::truncate_to_microseconds,
        types::{BookFormat, BookStore},
    },
    domain::error::DomainError,
    impl_string_value_object,
};

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
        write!(f, "{}", self.id.hyphenated())
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

static ISBN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(^$|^(\d-?){12}\d$)").expect("ISBN_REGEX is a hardcoded valid pattern")
});

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct Isbn {
    // TODO: Validate check digit
    #[validate(regex(path = *ISBN_REGEX))]
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
    created_at: OffsetDateTime,
    #[getset(get = "pub")]
    updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookUpdate {
    pub title: BookTitle,
    pub author_ids: Vec<AuthorId>,
    pub isbn: Isbn,
    pub read: ReadFlag,
    pub owned: OwnedFlag,
    pub priority: Priority,
    pub format: BookFormat,
    pub store: BookStore,
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
            created_at: truncate_to_microseconds(created_at),
            updated_at: truncate_to_microseconds(updated_at),
        })
    }

    pub fn update(&mut self, update: BookUpdate, updated_at: OffsetDateTime) {
        self.title = update.title;
        self.author_ids = update.author_ids;
        self.isbn = update.isbn;
        self.read = update.read;
        self.owned = update.owned;
        self.priority = update.priority;
        self.format = update.format;
        self.store = update.store;
        self.updated_at = truncate_to_microseconds(updated_at);
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
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::common::types::{BookFormat, BookStore};

    use super::{Book, BookId, BookTitle, BookUpdate, Isbn, OwnedFlag, Priority, ReadFlag};
    use crate::domain::entity::author::AuthorId;

    #[test]
    fn update_updates_editable_fields_and_updated_at() {
        let id = BookId::new(Uuid::new_v4()).expect("valid book id");
        let created_at = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid time");
        let original_updated_at =
            OffsetDateTime::from_unix_timestamp(1_700_000_100).expect("valid time");
        let updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_200).expect("valid time");
        let original_author_id = AuthorId::new(Uuid::new_v4());
        let updated_author_ids = vec![AuthorId::new(Uuid::new_v4()), AuthorId::new(Uuid::new_v4())];

        let mut book = Book::new(
            id.clone(),
            BookTitle::new("Original title".to_owned()).expect("valid title"),
            vec![original_author_id],
            Isbn::new("9784062758574".to_owned()).expect("valid isbn"),
            ReadFlag::new(false),
            OwnedFlag::new(false),
            Priority::new(10).expect("valid priority"),
            BookFormat::Printed,
            BookStore::Unknown,
            created_at,
            original_updated_at,
        )
        .expect("valid book");

        let update = BookUpdate {
            title: BookTitle::new("Updated title".to_owned()).expect("valid title"),
            author_ids: updated_author_ids.clone(),
            isbn: Isbn::new("978-4062758574".to_owned()).expect("valid isbn"),
            read: ReadFlag::new(true),
            owned: OwnedFlag::new(true),
            priority: Priority::new(99).expect("valid priority"),
            format: BookFormat::EBook,
            store: BookStore::Kindle,
        };

        book.update(update, updated_at);

        assert_eq!(book.title().as_str(), "Updated title");
        assert_eq!(book.author_ids(), &updated_author_ids);
        assert_eq!(book.isbn().as_str(), "978-4062758574");
        assert!(book.read().to_bool());
        assert!(book.owned().to_bool());
        assert_eq!(book.priority().to_i32(), 99);
        assert_eq!(book.format(), &BookFormat::EBook);
        assert_eq!(book.store(), &BookStore::Kindle);
        assert_eq!(book.updated_at(), &updated_at);
        assert_eq!(book.id(), &id);
        assert_eq!(book.created_at(), &created_at);
    }

    #[test]
    fn valid_isbn_with_hyphen() {
        let isbn = Isbn::new("978-4062758574".to_owned());
        assert!(isbn.is_ok());
    }

    #[test]
    fn valid_isbn_without_hyphen() {
        let isbn = Isbn::new("9784062758574".to_owned());
        assert!(isbn.is_ok());
    }

    #[test]
    fn empty_isbn_is_valid() {
        let isbn = Isbn::new("".to_owned());
        assert!(isbn.is_ok());
    }

    #[test]
    fn isbn_too_short() {
        let isbn = Isbn::new("1".to_owned());
        assert!(isbn.is_err());
    }

    #[test]
    fn priority_0_is_valid() {
        let priority = Priority::new(0);
        assert!(priority.is_ok());
    }

    #[test]
    fn priority_100_is_valid() {
        let priority = Priority::new(100);
        assert!(priority.is_ok());
    }

    #[test]
    fn priority_negative1_is_invalid() {
        let priority = Priority::new(-1);
        assert!(priority.is_err());
    }

    #[test]
    fn priority_101_is_invalid() {
        let priority = Priority::new(101);
        assert!(priority.is_err());
    }
}
