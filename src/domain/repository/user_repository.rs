use crate::domain::{entity::user::User, error::domain_error::DomainError};

pub trait UserRepository {
    fn create(user: User) -> Result<(), DomainError>;
    fn find_by_id() -> Result<Option<User>, DomainError>;
}
