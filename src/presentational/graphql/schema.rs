use async_graphql::{EmptySubscription, Object, Schema, EmptyMutation};

pub struct Query;

#[Object]
impl Query {
    async fn field1(&self) -> String {
        String::from("value1")
    }
}

fn build_schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}
