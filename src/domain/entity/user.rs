use getset::Getters;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct User {
    #[getset(get = "pub")]
    id: String,
}

impl User {
    pub fn new(id: String) -> User {
        User { id }
    }
}
