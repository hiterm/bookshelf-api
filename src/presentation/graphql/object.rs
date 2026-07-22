use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Enum, Json, Result};
use async_graphql::{ID, InputObject, SimpleObject};
use serde_json::Value;
use time::OffsetDateTime;

use crate::common::types::{BookFormat as CommonBookFormat, BookStore as CommonBookStore};
use crate::dependency_injection::QI;
use crate::use_case::dto::author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto};
use crate::use_case::dto::book::{BookDto, CreateBookDto, ImportBookEntryDto, UpdateBookDto};
use crate::use_case::dto::event::{AuthorEventDto, BookEventDto};
use crate::use_case::dto::event_set::{EventSetDetailDto, EventSetDto};

use super::loader::AuthorLoader;

#[derive(SimpleObject)]
pub struct User {
    id: ID,
}

impl User {
    pub fn new(id: ID) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum BookFormat {
    EBook,
    Printed,
    Unknown,
}

impl From<CommonBookFormat> for BookFormat {
    fn from(book_format: CommonBookFormat) -> Self {
        match book_format {
            CommonBookFormat::EBook => BookFormat::EBook,
            CommonBookFormat::Printed => BookFormat::Printed,
            CommonBookFormat::Unknown => BookFormat::Unknown,
        }
    }
}

impl From<BookFormat> for CommonBookFormat {
    fn from(book_format: BookFormat) -> Self {
        match book_format {
            BookFormat::EBook => CommonBookFormat::EBook,
            BookFormat::Printed => CommonBookFormat::Printed,
            BookFormat::Unknown => CommonBookFormat::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum BookStore {
    Kindle,
    Unknown,
}

impl From<CommonBookStore> for BookStore {
    fn from(book_format: CommonBookStore) -> Self {
        match book_format {
            CommonBookStore::Kindle => BookStore::Kindle,
            CommonBookStore::Unknown => BookStore::Unknown,
        }
    }
}

impl From<BookStore> for CommonBookStore {
    fn from(book_format: BookStore) -> Self {
        match book_format {
            BookStore::Kindle => CommonBookStore::Kindle,
            BookStore::Unknown => CommonBookStore::Unknown,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Book {
    pub id: String,
    pub title: String,
    #[graphql(skip)]
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Book {
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
        created_at: i64,
        updated_at: i64,
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
            created_at,
            updated_at,
        }
    }
}

#[ComplexObject]
impl Book {
    async fn authors(&self, ctx: &Context<'_>) -> Result<Vec<Author>> {
        // QIの型はGenericにできないか
        let loader = ctx.data_unchecked::<DataLoader<AuthorLoader<QI>>>();
        let authors: Vec<Author> = loader
            .load_many(self.author_ids.clone()) // TODO cloneやめる
            .await?
            .into_values()
            .collect();

        Ok(authors)
    }
}

impl From<BookDto> for Book {
    fn from(book_dto: BookDto) -> Self {
        Self {
            id: book_dto.id,
            title: book_dto.title,
            author_ids: book_dto.author_ids,
            isbn: book_dto.isbn,
            read: book_dto.read,
            owned: book_dto.owned,
            priority: book_dto.priority,
            format: book_dto.format.into(),
            store: book_dto.store.into(),
            created_at: book_dto.created_at.unix_timestamp(),
            updated_at: book_dto.updated_at.unix_timestamp(),
        }
    }
}

#[derive(InputObject)]
pub struct CreateBookInput {
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: BookFormat,
    pub store: BookStore,
}

impl From<CreateBookInput> for CreateBookDto {
    fn from(book_input: CreateBookInput) -> Self {
        let CreateBookInput {
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
        } = book_input;

        CreateBookDto::new(
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format.into(),
            store.into(),
        )
    }
}

#[derive(InputObject)]
pub struct UpdateBookInput {
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

impl From<UpdateBookInput> for UpdateBookDto {
    fn from(book_input: UpdateBookInput) -> Self {
        let UpdateBookInput {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
        } = book_input;

        UpdateBookDto::new(
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format.into(),
            store.into(),
        )
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct Author {
    pub id: ID,
    pub name: String,
    pub yomi: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Author {
    pub fn new(
        id: String,
        name: String,
        yomi: String,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Self {
        Self {
            id: ID(id),
            name,
            yomi,
            created_at,
            updated_at,
        }
    }
}

impl From<AuthorDto> for Author {
    fn from(author: AuthorDto) -> Self {
        let AuthorDto {
            id,
            name,
            yomi,
            created_at,
            updated_at,
        } = author;
        Author::new(id, name, yomi, created_at, updated_at)
    }
}

#[derive(InputObject)]
pub struct CreateAuthorInput {
    pub name: String,
    pub yomi: Option<String>,
}

impl CreateAuthorInput {
    pub fn new(name: String) -> Self {
        Self { name, yomi: None }
    }
}

impl From<CreateAuthorInput> for CreateAuthorDto {
    fn from(val: CreateAuthorInput) -> Self {
        CreateAuthorDto {
            name: val.name,
            yomi: val.yomi,
        }
    }
}

#[derive(InputObject)]
pub struct UpdateAuthorInput {
    pub id: ID,
    pub name: String,
    pub yomi: Option<String>,
}

impl From<UpdateAuthorInput> for UpdateAuthorDto {
    fn from(val: UpdateAuthorInput) -> Self {
        UpdateAuthorDto {
            id: val.id.to_string(),
            name: val.name,
            yomi: val.yomi,
        }
    }
}

#[derive(InputObject)]
pub struct ImportBookInput {
    /// Title of the book.
    pub title: String,
    /// Names of the authors. Authors will be created if they do not exist.
    pub author_names: Vec<String>,
    /// ISBN of the book.
    pub isbn: String,
    /// Whether the book has been read.
    pub read: bool,
    /// Whether the book is owned.
    pub owned: bool,
    /// Priority value ranging from 0 to 100.
    pub priority: i32,
    /// Format of the book.
    pub format: BookFormat,
    /// Store where the book was purchased or obtained.
    pub store: BookStore,
}

impl From<ImportBookInput> for ImportBookEntryDto {
    fn from(input: ImportBookInput) -> Self {
        ImportBookEntryDto {
            title: input.title,
            author_names: input.author_names,
            isbn: input.isbn,
            read: input.read,
            owned: input.owned,
            priority: input.priority,
            format: input.format.into(),
            store: input.store.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct BookEventEntry {
    pub event_id: ID,
    pub event_set_id: ID,
    pub operation: String,
    pub book_id: ID,
    pub title: Option<String>,
    pub author_ids: Vec<ID>,
    pub isbn: Option<String>,
    pub read: Option<bool>,
    pub owned: Option<bool>,
    pub priority: Option<i32>,
    pub format: Option<BookFormat>,
    pub store: Option<BookStore>,
    pub book_created_at: Option<i64>,
    pub book_updated_at: Option<i64>,
    pub changed_at: i64,
    pub extra: Option<Json<Value>>,
}

impl From<BookEventDto> for BookEventEntry {
    fn from(dto: BookEventDto) -> Self {
        Self {
            event_id: ID(dto.event_id.to_string()),
            event_set_id: ID(dto.event_set_id),
            operation: dto.operation,
            book_id: ID(dto.book_id),
            title: dto.title,
            author_ids: dto.author_ids.into_iter().map(ID).collect(),
            isbn: dto.isbn,
            read: dto.read,
            owned: dto.owned,
            priority: dto.priority,
            format: dto.format.map(Into::into),
            store: dto.store.map(Into::into),
            book_created_at: dto.book_created_at.map(|t| t.unix_timestamp()),
            book_updated_at: dto.book_updated_at.map(|t| t.unix_timestamp()),
            changed_at: dto.changed_at.unix_timestamp(),
            extra: dto.extra.map(Json),
        }
    }
}

#[derive(SimpleObject)]
pub struct AuthorEventEntry {
    pub event_id: ID,
    pub event_set_id: ID,
    pub operation: String,
    pub author_id: ID,
    pub name: Option<String>,
    pub yomi: Option<String>,
    pub author_created_at: Option<i64>,
    pub author_updated_at: Option<i64>,
    pub changed_at: i64,
    pub extra: Option<Json<Value>>,
}

impl From<AuthorEventDto> for AuthorEventEntry {
    fn from(dto: AuthorEventDto) -> Self {
        Self {
            event_id: ID(dto.event_id.to_string()),
            event_set_id: ID(dto.event_set_id),
            operation: dto.operation,
            author_id: ID(dto.author_id),
            name: dto.name,
            yomi: dto.yomi,
            author_created_at: dto.author_created_at.map(|t| t.unix_timestamp()),
            author_updated_at: dto.author_updated_at.map(|t| t.unix_timestamp()),
            changed_at: dto.changed_at.unix_timestamp(),
            extra: dto.extra.map(Json),
        }
    }
}

#[derive(SimpleObject)]
pub struct EventSetEntry {
    pub id: ID,
    pub operation: String,
    pub created_at: i64,
}

impl From<EventSetDto> for EventSetEntry {
    fn from(dto: EventSetDto) -> Self {
        Self {
            id: ID(dto.id),
            operation: dto.operation,
            created_at: dto.created_at.unix_timestamp(),
        }
    }
}

#[derive(SimpleObject)]
pub struct EventSetDetail {
    pub id: ID,
    pub operation: String,
    pub created_at: i64,
    pub book_events: Vec<BookEventEntry>,
    pub author_events: Vec<AuthorEventEntry>,
}

impl From<EventSetDetailDto> for EventSetDetail {
    fn from(dto: EventSetDetailDto) -> Self {
        Self {
            id: ID(dto.id),
            operation: dto.operation,
            created_at: dto.created_at.unix_timestamp(),
            book_events: dto
                .book_events
                .into_iter()
                .map(BookEventEntry::from)
                .collect(),
            author_events: dto
                .author_events
                .into_iter()
                .map(AuthorEventEntry::from)
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct BookMutationPayload {
    pub book: Book,
    pub event_set_id: ID,
}

impl BookMutationPayload {
    pub fn new(book: Book, event_set_id: ID) -> Self {
        Self { book, event_set_id }
    }
}

#[derive(SimpleObject)]
pub struct AuthorMutationPayload {
    pub author: Author,
    pub event_set_id: ID,
}

impl AuthorMutationPayload {
    pub fn new(author: Author, event_set_id: ID) -> Self {
        Self {
            author,
            event_set_id,
        }
    }
}

#[derive(SimpleObject)]
pub struct DeleteBookPayload {
    pub book_id: ID,
    pub event_set_id: ID,
}

#[derive(SimpleObject)]
pub struct DeleteAuthorPayload {
    pub author_id: ID,
    pub event_set_id: ID,
}

#[derive(SimpleObject)]
pub struct ImportBooksPayload {
    pub books: Vec<Book>,
    pub event_set_id: ID,
}

#[derive(SimpleObject)]
pub struct RestoreBookPayload {
    pub book: Option<Book>,
    pub event_set_id: ID,
}

#[derive(SimpleObject)]
pub struct RestoreAuthorPayload {
    pub author: Option<Author>,
    pub event_set_id: ID,
}
