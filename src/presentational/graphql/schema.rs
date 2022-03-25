use crate::use_case::use_case::query::QueryUseCase;

use super::query::Query;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

pub fn build_schema<T>(query: Query<T>) -> Schema<Query<T>, EmptyMutation, EmptySubscription>
where
    T: QueryUseCase,
{
    Schema::build(query, EmptyMutation, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {
    // use mockall::predicate;

    // use crate::{
    //     presentational::graphql::query::Query,
    //     use_case::{dto::author::Author, use_case::query::MockQueryUseCase},
    // };

    // use super::build_schema;

    // #[tokio::test]
    // async fn execute_query() {
    //     let user_id = "user1";
    //     let author_id = "d065a358-4fa7-4236-ae19-f6f2f9467c35";
    //     let mut mock_query_use_case = MockQueryUseCase::new();
    //     mock_query_use_case
    //         .expect_find_author_by_id()
    //         .with(predicate::eq(user_id), predicate::eq(author_id))
    //         .times(1)
    //         .returning(|_user_id, author_id| {
    //             Ok(Author {
    //                 id: author_id.to_string(),
    //                 name: String::from("author1"),
    //             })
    //         });
    //     let query = Query::new(mock_query_use_case);
    //     let schema = build_schema(query);
    //     let res = schema
    //         .execute(r#"query { author(id: "d065a358-4fa7-4236-ae19-f6f2f9467c35") {id, name} }"#)
    //         .await;
    //     let json = serde_json::to_value(&res).unwrap();
    //     // assert_eq!(json, "a");
    //     assert_eq!(json, "author1");
    //     // assert_eq!(json["data"]["author"]["name"], "author1");
    // }
}
