use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

use super::{query::QueryRoot, query_service::QueryService};

pub struct Query;

#[Object]
impl Query {
    async fn field1(&self) -> String {
        String::from("value1")
    }
}

fn build_schema<T>(query: QueryRoot<T>) -> Schema<QueryRoot<T>, EmptyMutation, EmptySubscription>
where
    T: QueryService + Send + Sync + 'static,
{
    Schema::build(query, EmptyMutation, EmptySubscription).finish()
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
