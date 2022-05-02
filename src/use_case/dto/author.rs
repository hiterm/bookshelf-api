use crate::domain::entity::author::{Author, DestructureAuthor};

#[derive(Debug, PartialEq, Eq)]
pub struct AuthorDto {
    pub id: String,
    pub name: String,
}

impl From<Author> for AuthorDto {
    fn from(author: Author) -> Self {
        let DestructureAuthor { id, name } = author.destructure();
        AuthorDto {
            id: id.to_string(),
            name: name.into_string(),
        }
    }
}

pub struct CreateAuthorDto {
    pub name: String,
}

impl CreateAuthorDto {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
