use getset::Getters;

use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct UserId {
    #[getset(get = "pub")]
    pub id: String,
}

impl UserId {
    pub fn new(id: String) -> Result<UserId, DomainError> {
        Ok(UserId { id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct User {
    #[getset(get = "pub")]
    pub id: UserId,
}

impl User {
    pub fn new(id: UserId) -> User {
        User { id }
    }
}
