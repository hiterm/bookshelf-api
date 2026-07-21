use crate::use_case::traits::{mutation::MutationUseCase, query::QueryUseCase};

use super::{mutation::Mutation, query::Query};
use async_graphql::{EmptySubscription, Schema};

pub fn build_schema<QUC, MUC>(
    query: Query<QUC>,
    mutation: Mutation<MUC>,
) -> Schema<Query<QUC>, Mutation<MUC>, EmptySubscription>
where
    QUC: QueryUseCase,
    MUC: MutationUseCase,
{
    Schema::build(query, mutation, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {
    use mockall::predicate;

    use crate::{
        presentation::{
            extractor::claims::Claims,
            graphql::{mutation::Mutation, query::Query},
        },
        use_case::{
            dto::author::AuthorDto,
            traits::{mutation::MockMutationUseCase, query::MockQueryUseCase},
        },
    };

    use super::build_schema;

    #[tokio::test]
    async fn execute_query() {
        let user_id = "user1";
        let author_id = "d065a358-4fa7-4236-ae19-f6f2f9467c35";
        let author_name = "author1";

        let mut mock_query_use_case = MockQueryUseCase::new();
        mock_query_use_case
            .expect_find_author_by_id()
            .with(predicate::eq(user_id), predicate::eq(author_id))
            .times(1)
            .returning(|_user_id, author_id| {
                Ok(Some(AuthorDto {
                    id: author_id.to_string(),
                    name: author_name.to_string(),
                    yomi: "おーさーわん".to_string(),
                    created_at: time::OffsetDateTime::UNIX_EPOCH,
                    updated_at: time::OffsetDateTime::UNIX_EPOCH,
                }))
            });
        let query = Query::new(mock_query_use_case);
        let mutation_use_case = MockMutationUseCase::new();
        let mutation = Mutation::new(mutation_use_case);
        let schema = build_schema(query, mutation);
        let claims = Claims {
            sub: user_id.to_string(),
            _permissions: None,
        };
        let res = schema
            .execute(
                async_graphql::Request::from(
                    r#"query { author(id: "d065a358-4fa7-4236-ae19-f6f2f9467c35") {id, name, yomi, createdAt, updatedAt} }"#,
                )
                .data(claims),
            )
            .await;
        let json = serde_json::to_value(&res).unwrap();
        assert_eq!(json["data"]["author"]["name"], author_name);
        assert_eq!(json["data"]["author"]["yomi"], "おーさーわん");
        assert_eq!(json["data"]["author"]["createdAt"], 0);
        assert_eq!(json["data"]["author"]["updatedAt"], 0);
    }

    #[test]
    fn mutation_payloads_expose_only_canonical_fields() {
        let query = Query::new(MockQueryUseCase::new());
        let mutation = Mutation::new(MockMutationUseCase::new());
        let sdl = build_schema(query, mutation).sdl();

        assert!(sdl.contains("type BookMutationPayload {\n\tbook: Book!\n\teventSetId: ID!\n}"));
        assert!(
            sdl.contains("type AuthorMutationPayload {\n\tauthor: Author!\n\teventSetId: ID!\n}")
        );
        assert!(sdl.contains("type DeleteBookPayload {\n\tbookId: ID!\n\teventSetId: ID!\n}"));
        assert!(sdl.contains("type DeleteAuthorPayload {\n\tauthorId: ID!\n\teventSetId: ID!\n}"));
    }
}
