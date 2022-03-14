use getset::Getters;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct Author {
    #[getset(get = "pub")]
    id: Uuid,
    #[getset(get = "pub")]
    name: String,
}

impl Author {
    pub fn new(id: Uuid, name: String) -> Author {
        Author { id, name }
    }
}
