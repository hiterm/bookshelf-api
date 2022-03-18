use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

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

#[cfg(test)]
mod tests {
    use super::build_schema;

    #[tokio::test]
    async fn execute_query() {
        let schema = build_schema();
        let res = schema.execute("{ field1 }").await;
        let json = serde_json::to_value(&res).unwrap();
        assert_eq!(json["data"]["field1"], "value1");
    }
}
