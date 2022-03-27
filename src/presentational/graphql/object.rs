use async_graphql::{InputObject, SimpleObject, ID};

use crate::use_case::dto::author::Author as AuthorDto;
use crate::use_case::dto::author::CreateAuthorData as UseCaseCreateAuthorData;

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
pub struct CreateAuthorData {
    pub name: String,
}

impl CreateAuthorData {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Into<UseCaseCreateAuthorData> for CreateAuthorData {
    fn into(self) -> UseCaseCreateAuthorData {
        UseCaseCreateAuthorData::new(self.name)
    }
}
