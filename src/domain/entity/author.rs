use uuid::Uuid;
use validator::Validate;

use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorId {
    pub id: Uuid,
}

impl AuthorId {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

impl TryFrom<&str> for AuthorId {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(value).map_err(|err| {
            DomainError::Validation(format!(
                r#"Failed to parse id "{}" as uuid. Message from uuid crate: {}"#,
                value,
                err.to_string()
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
    pub name: String,
}

impl AuthorName {
    pub fn new(name: String) -> Result<AuthorName, DomainError> {
        let author_name = AuthorName { name };
        author_name.validate()?;
        Ok(author_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Author {
    pub id: AuthorId,
    pub name: AuthorName,
}

impl Author {
    pub fn new(id: AuthorId, name: AuthorName) -> Result<Author, DomainError> {
        Ok(Author { id, name })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{entity::author::AuthorName, error::DomainError};

    #[test]
    fn validation_success() {
        assert!(matches!(AuthorName::new(String::from("author1")), Ok(_)));
    }

    #[test]
    fn validation_failure() {
        assert!(matches!(
            AuthorName::new(String::from("")),
            Err(DomainError::Validation(_))
        ));
    }
}
