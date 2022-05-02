use crate::domain::entity::user::User as DomainUser;

pub struct UserDto {
    pub id: String,
}

impl UserDto {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl From<DomainUser> for UserDto {
    fn from(user: DomainUser) -> Self {
        UserDto::new(user.id.into_string())
    }
}
