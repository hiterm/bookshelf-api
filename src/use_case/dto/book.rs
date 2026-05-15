use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::AuthorId,
            book::{Book, BookId, BookTitle, DestructureBook, Isbn, OwnedFlag, Priority, ReadFlag},
        },
        error::DomainError,
    },
    use_case::error::UseCaseError,
};

#[derive(Debug, Clone)]
pub struct BookDto {
    pub id: String,
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Book> for BookDto {
    fn from(book: Book) -> Self {
        let DestructureBook {
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
        } = book.destructure();

        Self {
            id: id.to_string(),
            title: title.into_string(),
            author_ids: author_ids
                .into_iter()
                .map(|author_id| author_id.to_string())
                .collect(),
            isbn: isbn.into_string(),
            read: read.to_bool(),
            owned: owned.to_bool(),
            priority: priority.to_i32(),
            format,
            store,
            created_at,
            updated_at,
        }
    }
}

pub struct TimeInfo {
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl TimeInfo {
    pub fn new(created_at: OffsetDateTime, updated_at: OffsetDateTime) -> Self {
        Self {
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateBookDto {
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
}

impl CreateBookDto {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        author_ids: Vec<String>,
        isbn: String,
        read: bool,
        owned: bool,
        priority: i32,
        format: BookFormat,
        store: BookStore,
    ) -> Self {
        Self {
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
        }
    }
}

impl TryFrom<(Uuid, CreateBookDto, TimeInfo)> for Book {
    type Error = UseCaseError;

    fn try_from(
        (uuid, book_data, time_info): (Uuid, CreateBookDto, TimeInfo),
    ) -> Result<Self, Self::Error> {
        let author_ids: Result<Vec<AuthorId>, DomainError> = book_data
            .author_ids
            .into_iter()
            .map(|author_id| AuthorId::try_from(author_id.as_str()))
            .collect();
        let author_ids = author_ids?;

        let book = Book::new(
            BookId::new(uuid)?,
            BookTitle::new(book_data.title)?,
            author_ids,
            Isbn::new(book_data.isbn)?,
            ReadFlag::new(book_data.read),
            OwnedFlag::new(book_data.owned),
            Priority::new(book_data.priority)?,
            book_data.format,
            book_data.store,
            time_info.created_at,
            time_info.updated_at,
        )?;

        Ok(book)
    }
}

#[derive(Debug, Clone)]
pub struct UpdateBookDto {
    pub id: String,
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
}

#[derive(Debug, Clone)]
pub struct ImportBookEntryDto {
    pub title: String,
    pub author_names: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
}

impl UpdateBookDto {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        title: String,
        author_ids: Vec<String>,
        isbn: String,
        read: bool,
        owned: bool,
        priority: i32,
        format: BookFormat,
        store: BookStore,
    ) -> Self {
        Self {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
        }
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::entity::{
            author::AuthorId,
            book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
        },
    };

    use super::{BookDto, CreateBookDto, TimeInfo};

    #[test]
    fn book_dto_from_book_maps_all_fields() {
        // Given
        let uuid_str = "a1b2c3d4-e5f6-4890-abcd-ef1234567890";
        let uuid = Uuid::parse_str(uuid_str).unwrap();
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let now = OffsetDateTime::now_utc();

        let book = Book::new(
            BookId::new(uuid).unwrap(),
            BookTitle::new("My Book".to_string()).unwrap(),
            vec![AuthorId::try_from(author_id_str).unwrap()],
            Isbn::new("9784062758574".to_string()).unwrap(),
            ReadFlag::new(true),
            OwnedFlag::new(false),
            Priority::new(80).unwrap(),
            BookFormat::EBook,
            BookStore::Kindle,
            now,
            now,
        )
        .unwrap();

        // When
        let dto = BookDto::from(book);

        // Then
        assert_eq!(dto.id, uuid_str);
        assert_eq!(dto.title, "My Book");
        assert_eq!(dto.author_ids, vec![author_id_str]);
        assert_eq!(dto.isbn, "9784062758574");
        assert!(dto.read);
        assert!(!dto.owned);
        assert_eq!(dto.priority, 80);
        assert_eq!(dto.format, BookFormat::EBook);
        assert_eq!(dto.store, BookStore::Kindle);
    }

    #[test]
    fn book_try_from_create_dto_success() {
        // Given
        let uuid = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let time_info = TimeInfo::new(now, now);
        let create_dto = CreateBookDto::new(
            "New Book".to_string(),
            vec![],
            "".to_string(),
            false,
            true,
            30,
            BookFormat::Printed,
            BookStore::Unknown,
        );

        // When
        let result = Book::try_from((uuid, create_dto, time_info));

        // Then
        assert!(result.is_ok());
        let book = result.unwrap();
        assert_eq!(book.title().as_str(), "New Book");
        assert_eq!(book.priority().to_i32(), 30);
        assert!(book.owned().to_bool());
    }

    #[test]
    fn book_try_from_create_dto_fails_with_empty_title() {
        // Given
        let uuid = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let time_info = TimeInfo::new(now, now);
        let create_dto = CreateBookDto::new(
            "".to_string(),
            vec![],
            "".to_string(),
            false,
            false,
            0,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = Book::try_from((uuid, create_dto, time_info));

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn book_try_from_create_dto_fails_with_invalid_isbn() {
        // Given
        let uuid = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let time_info = TimeInfo::new(now, now);
        let create_dto = CreateBookDto::new(
            "Valid Title".to_string(),
            vec![],
            "1".to_string(),
            false,
            false,
            0,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = Book::try_from((uuid, create_dto, time_info));

        // Then
        assert!(result.is_err());
    }
}
