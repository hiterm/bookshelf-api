use validator::Validate;

use crate::{domain::error::DomainError, impl_string_value_object};

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct UserId {
    #[validate(length(min = 1))]
    value: String,
}

impl_string_value_object!(UserId);

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
