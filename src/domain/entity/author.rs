use uuid::Uuid;
use validator::Validate;

use crate::{domain::error::DomainError, impl_string_value_object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorId {
    id: Uuid,
}

impl AuthorId {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    pub fn as_uuid(&self) -> Uuid {
        self.id
    }

    pub fn to_string(&self) -> String {
        self.id.to_hyphenated().to_string()
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
    value: String,
}

impl_string_value_object!(AuthorName);

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
    use uuid::Uuid;

    use crate::domain::{
        entity::author::{AuthorId, AuthorName},
        error::DomainError,
    };

    #[test]
    fn author_id_to_string() {
        let uuid_str = "c6ea22c8-7b70-470c-a713-c7aade5693bd";
        let uuid = Uuid::parse_str(uuid_str).unwrap();
        let author_id = AuthorId::new(uuid);
        assert_eq!(author_id.to_string(), uuid_str);
    }

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
