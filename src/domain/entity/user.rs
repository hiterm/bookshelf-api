use uuid::Uuid;

pub struct User {
    id: Uuid,
    sub: String,
}

impl User {
    pub fn new(id: Uuid, sub: String) -> User {
        User { id, sub }
    }
}
