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
