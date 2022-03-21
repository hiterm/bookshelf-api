use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct Book {
    id: String,
    title: String,
}

impl Book {
    pub fn new(id: String, title: String) -> Self {
        Book { id, title }
    }
}

#[derive(SimpleObject)]
pub struct Author {
    pub id: String,
    pub name: String,
}

impl Author {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}
