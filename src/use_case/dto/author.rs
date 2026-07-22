use crate::domain::entity::author::{Author, DestructureAuthor};
use time::OffsetDateTime;

#[derive(Debug, PartialEq, Eq)]
pub struct AuthorDto {
    pub id: String,
    pub name: String,
    pub yomi: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<Author> for AuthorDto {
    fn from(author: Author) -> Self {
        let DestructureAuthor {
            id,
            name,
            yomi,
            created_at,
            updated_at,
        } = author.destructure();
        AuthorDto {
            id: id.to_string(),
            name: name.into_string(),
            yomi,
            created_at,
            updated_at,
        }
    }
}

pub struct CreateAuthorDto {
    pub name: String,
    pub yomi: Option<String>,
}

impl CreateAuthorDto {
    pub fn new(name: String) -> Self {
        Self { name, yomi: None }
    }
}

pub struct UpdateAuthorDto {
    pub id: String,
    pub name: String,
    pub yomi: Option<String>,
}

impl UpdateAuthorDto {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            yomi: None,
        }
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
        let timestamp = time::OffsetDateTime::UNIX_EPOCH;
        let author = Author::new_with_timestamps(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Test Author".to_string()).unwrap(),
            String::new(),
            timestamp,
            timestamp,
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
                yomi: String::new(),
                created_at: timestamp,
                updated_at: timestamp,
            }
        );
    }
}
