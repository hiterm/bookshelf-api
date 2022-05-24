use async_graphql::dataloader::DataLoader;
use async_graphql::{ComplexObject, Context, Enum, Result};
use async_graphql::{InputObject, SimpleObject, ID};

use crate::dependency_injection::QI;
use crate::domain;
use crate::use_case::dto::author::AuthorDto;
use crate::use_case::dto::author::CreateAuthorDto;
use crate::use_case::dto::book::{BookDto, CreateBookDto, UpdateBookDto};
use domain::entity::book::{BookFormat as DomainBookFormat, BookStore as DomainBookStore};

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

impl From<DomainBookFormat> for BookFormat {
    fn from(book_format: DomainBookFormat) -> Self {
        match book_format {
            DomainBookFormat::EBook => BookFormat::EBook,
            DomainBookFormat::Printed => BookFormat::Printed,
            DomainBookFormat::Unknown => BookFormat::Unknown,
        }
    }
}

impl From<BookFormat> for DomainBookFormat {
    fn from(book_format: BookFormat) -> Self {
        match book_format {
            BookFormat::EBook => DomainBookFormat::EBook,
            BookFormat::Printed => DomainBookFormat::Printed,
            BookFormat::Unknown => DomainBookFormat::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum BookStore {
    Kindle,
    Unknown,
}

impl From<DomainBookStore> for BookStore {
    fn from(book_format: DomainBookStore) -> Self {
        match book_format {
            DomainBookStore::Kindle => BookStore::Kindle,
            DomainBookStore::Unknown => BookStore::Unknown,
        }
    }
}

impl From<BookStore> for DomainBookStore {
    fn from(book_format: BookStore) -> Self {
        match book_format {
            BookStore::Kindle => DomainBookStore::Kindle,
            BookStore::Unknown => DomainBookStore::Unknown,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Book {
    pub id: String,
    pub title: String,
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
}

impl Author {
    pub fn new(id: String, name: String) -> Self {
        Self { id: ID(id), name }
    }
}

impl From<AuthorDto> for Author {
    fn from(author: AuthorDto) -> Self {
        let AuthorDto { id, name } = author;
        Author::new(id, name)
    }
}

#[derive(InputObject)]
pub struct CreateAuthorInput {
    pub name: String,
}

impl CreateAuthorInput {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<CreateAuthorInput> for CreateAuthorDto {
    fn from(val: CreateAuthorInput) -> Self {
        CreateAuthorDto::new(val.name)
    }
}
