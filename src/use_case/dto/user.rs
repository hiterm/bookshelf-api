use crate::domain::entity::user::User as DomainUser;

pub struct User {
    pub id: String,
}

impl User {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl From<DomainUser> for User {
    fn from(user: DomainUser) -> Self {
        User::new(user.id.get_value())
    }
}
