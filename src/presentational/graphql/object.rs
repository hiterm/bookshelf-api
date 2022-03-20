use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct Book {
    id: String,
}
