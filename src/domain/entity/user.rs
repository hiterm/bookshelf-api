use validator::Validate;

use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct UserId {
    #[validate(length(min = 1))]
    pub value: String,
}

impl UserId {
    pub fn new(id: String) -> Result<UserId, DomainError> {
        let user_id = UserId { value: id };
        user_id.validate()?;
        Ok(user_id)
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
}

impl User {
    pub fn new(id: UserId) -> User {
        User { id }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::error::DomainError;

    use super::UserId;

    #[test]
    fn validation_success() {
        assert!(matches!(UserId::new(String::from("user1")), Ok(_)));
    }

    #[test]
    fn validation_failure() {
        assert!(matches!(
            UserId::new(String::from("")),
            Err(DomainError::Validation(_))
        ));
    }
}
