use super::{query::QueryRoot, query_service::QueryService};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

pub fn build_schema<T>(
    query: QueryRoot<T>,
) -> Schema<QueryRoot<T>, EmptyMutation, EmptySubscription>
where
    T: QueryService + Send + Sync + 'static,
{
    Schema::build(query, EmptyMutation, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {
    use mockall::predicate;

    use crate::{
        presentational::graphql::{query::QueryRoot, query_service::tests::MockQueryService},
        use_case::dto::book::Book,
    };

    use super::build_schema;

    #[tokio::test]
    async fn execute_query() {
        let id = "d065a358-4fa7-4236-ae19-f6f2f9467c35";
        let mut mock_query_service = MockQueryService::new();
        mock_query_service
            .expect_find_book_by_id()
            .with(predicate::eq(id))
            .times(1)
            .returning(|_id| {
                Ok(Book {
                    id: id.to_string(),
                    title: String::from("title1"),
                })
            });
        let query = QueryRoot::new(mock_query_service);
        let schema = build_schema(query);
        let res = schema
            .execute(r#"{ book(id: "d065a358-4fa7-4236-ae19-f6f2f9467c35") {id, title} }"#)
            .await;
        let json = serde_json::to_value(&res).unwrap();
        // assert_eq!(json, "a");
        assert_eq!(json["data"]["book"]["title"], "title1");
    }
}
