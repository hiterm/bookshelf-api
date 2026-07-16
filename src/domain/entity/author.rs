use std::fmt::Display;

use getset::Getters;
use uuid::Uuid;
use validator::Validate;

use crate::{domain::error::DomainError, impl_string_value_object};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorId {
    id: Uuid,
}

impl AuthorId {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    pub fn to_uuid(&self) -> Uuid {
        self.id
    }
}

impl Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id.hyphenated())
    }
}

impl TryFrom<&str> for AuthorId {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(value).map_err(|err| {
            DomainError::Validation(format!(
                r#"Failed to parse id "{}" as uuid. Message from uuid crate: {}"#,
                value, err
            ))
        })?;
        Ok(AuthorId { id })
    }
}

impl From<Uuid> for AuthorId {
    fn from(uuid: Uuid) -> Self {
        AuthorId { id: uuid }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct AuthorName {
    #[validate(length(min = 1))]
    value: String,
}

impl_string_value_object!(AuthorName);

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct Author {
    #[getset(get = "pub")]
    id: AuthorId,
    #[getset(get = "pub")]
    name: AuthorName,
    #[getset(get = "pub")]
    yomi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestructureAuthor {
    pub id: AuthorId,
    pub name: AuthorName,
    pub yomi: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorUpdate {
    pub name: AuthorName,
}

impl Author {
    pub fn new(id: AuthorId, name: AuthorName) -> Result<Author, DomainError> {
        Self::new_with_yomi(id, name, String::new())
    }

    pub fn new_with_yomi(
        id: AuthorId,
        name: AuthorName,
        yomi: String,
    ) -> Result<Author, DomainError> {
        Ok(Author { id, name, yomi })
    }

    pub fn update(&mut self, update: AuthorUpdate) {
        self.name = update.name;
    }

    pub fn destructure(self) -> DestructureAuthor {
        DestructureAuthor {
            id: self.id,
            name: self.name,
            yomi: self.yomi,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{
        entity::author::{Author, AuthorId, AuthorName, AuthorUpdate},
        error::DomainError,
    };

    #[test]
    fn author_id_to_string() {
        let uuid_str = "c6ea22c8-7b70-470c-a713-c7aade5693bd";
        let author_id = AuthorId::try_from(uuid_str).unwrap();
        assert_eq!(author_id.to_string(), uuid_str);
    }

    #[test]
    fn update_changes_name() {
        let mut author = Author::new(
            AuthorId::try_from("c6ea22c8-7b70-470c-a713-c7aade5693bd").unwrap(),
            AuthorName::new(String::from("author1")).unwrap(),
        )
        .unwrap();

        author.update(AuthorUpdate {
            name: AuthorName::new(String::from("author2")).unwrap(),
        });

        assert_eq!(author.name().as_str(), "author2");
    }

    #[test]
    fn validation_success() {
        assert!(AuthorName::new(String::from("author1")).is_ok());
    }

    #[test]
    fn validation_failure() {
        assert!(matches!(
            AuthorName::new(String::from("")),
            Err(DomainError::Validation(_))
        ));
    }
}
