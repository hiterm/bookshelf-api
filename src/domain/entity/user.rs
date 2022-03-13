use getset::Getters;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct User {
    #[getset(get = "pub")]
    id: Uuid,
    #[getset(get = "pub")]
    sub: String,
}

impl User {
    pub fn new(id: Uuid, sub: String) -> User {
        User { id, sub }
    }
}
