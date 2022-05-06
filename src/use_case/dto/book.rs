use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::AuthorId,
            book::{
                Book, BookFormat, BookId, BookStore, BookTitle, DestructureBook, Isbn, OwnedFlag,
                Priority, ReadFlag,
            },
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
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl CreateBookDto {
    pub fn new(
        title: String,
        author_ids: Vec<String>,
        isbn: String,
        read: bool,
        owned: bool,
        priority: i32,
        format: BookFormat,
        store: BookStore,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
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
            created_at,
            updated_at,
        }
    }
}

impl TryFrom<(Uuid, CreateBookDto)> for Book {
    type Error = UseCaseError;

    fn try_from((uuid, book_data): (Uuid, CreateBookDto)) -> Result<Self, Self::Error> {
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
            book_data.created_at,
            book_data.updated_at,
        )?;

        Ok(book)
    }
}
