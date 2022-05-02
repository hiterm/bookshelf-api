use async_graphql::{InputObject, SimpleObject, ID};

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

// TODO
#[derive(SimpleObject)]
pub struct Book {
    id: String,
    title: String,
}

// TODO
impl Book {
    pub fn new(id: String, title: String) -> Self {
        Book { id, title }
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
