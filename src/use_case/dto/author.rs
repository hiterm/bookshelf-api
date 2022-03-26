use crate::domain;

pub struct Author {
    pub id: String,
    pub name: String,
}

impl From<domain::entity::author::Author> for Author {
    fn from(author: domain::entity::author::Author) -> Self {
        let domain::entity::author::Author { id, name } = author;
        Author {
            id: id.id.to_string(),
            name: name.name,
        }
    }
}

pub struct CreateAuthorData {
    pub name: String,
}

impl CreateAuthorData {
    pub fn new(name: String) -> Self { Self { name } }
}
