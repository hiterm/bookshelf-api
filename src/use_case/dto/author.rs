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

#[cfg(test)]
mod tests {
    use crate::domain::entity::author::{Author, AuthorId, AuthorName};

    use super::AuthorDto;

    #[test]
    fn author_dto_from_author() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author = Author::new(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Test Author".to_string()).unwrap(),
        )
        .unwrap();

        // When
        let dto = AuthorDto::from(author);

        // Then
        assert_eq!(
            dto,
            AuthorDto {
                id: author_id_str.to_string(),
                name: "Test Author".to_string(),
            }
        );
    }
}
