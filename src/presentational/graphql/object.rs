use async_graphql::Enum;
use async_graphql::{InputObject, SimpleObject, ID};
use time::PrimitiveDateTime;

use crate::use_case::dto::author::AuthorDto;
use crate::use_case::dto::author::CreateAuthorDto;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum BookStore {
    Kindle,
    Unknown,
}

#[derive(SimpleObject)]
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
    pub created_at: u64,
    pub updated_at: u64,
}

impl Book {
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
        created_at: u64,
        updated_at: u64,
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

#[derive(SimpleObject)]
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

impl Into<CreateAuthorDto> for CreateAuthorInput {
    fn into(self) -> CreateAuthorDto {
        CreateAuthorDto::new(self.name)
    }
}
