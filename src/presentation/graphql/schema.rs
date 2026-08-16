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
            dto::{author::AuthorDto, mutation::SingleEventMutationResultDto},
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
        assert_eq!(json["data"]["author"]["createdAt"], "1970-01-01T00:00:00Z");
        assert_eq!(json["data"]["author"]["updatedAt"], "1970-01-01T00:00:00Z");
    }

    #[tokio::test]
    async fn create_author_payload_exposes_numeric_event_id_as_graphql_id() {
        let mut mutation_use_case = MockMutationUseCase::new();
        mutation_use_case
            .expect_create_author()
            .with(predicate::eq("user1"), predicate::always())
            .returning(|_, input| {
                Ok(SingleEventMutationResultDto::new(
                    AuthorDto {
                        id: "d065a358-4fa7-4236-ae19-f6f2f9467c35".to_string(),
                        name: input.name,
                        yomi: String::new(),
                        created_at: time::OffsetDateTime::UNIX_EPOCH,
                        updated_at: time::OffsetDateTime::UNIX_EPOCH,
                    },
                    "e77df9d5-b7bf-47f2-8753-03f285d440e3".to_string(),
                    1234.into(),
                ))
            });
        let schema = build_schema(
            Query::new(MockQueryUseCase::new()),
            Mutation::new(mutation_use_case),
        );
        let claims = Claims {
            sub: "user1".to_string(),
            _permissions: None,
        };

        let response = schema
            .execute(
                async_graphql::Request::from(
                    r#"mutation { createAuthor(authorData: { name: "Author" }) { eventId eventSetId author { name } } }"#,
                )
                .data(claims),
            )
            .await;
        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json["data"]["createAuthor"]["eventId"], "1234");
        assert_eq!(
            json["data"]["createAuthor"]["eventSetId"],
            "e77df9d5-b7bf-47f2-8753-03f285d440e3"
        );
        assert_eq!(json["data"]["createAuthor"]["author"]["name"], "Author");
    }

    #[test]
    fn mutation_payloads_expose_only_canonical_fields() {
        let query = Query::new(MockQueryUseCase::new());
        let mutation = Mutation::new(MockMutationUseCase::new());
        let sdl = build_schema(query, mutation).sdl();

        assert!(sdl.contains(
            "type BookMutationPayload {\n\tbook: Book!\n\teventSetId: ID!\n\teventId: ID!\n}"
        ));
        assert!(sdl.contains(
            "type AuthorMutationPayload {\n\tauthor: Author!\n\teventSetId: ID!\n\teventId: ID!\n}"
        ));
        assert!(sdl.contains("type DeleteBookPayload {\n\tbookId: ID!\n\teventSetId: ID!\n}"));
        assert!(sdl.contains("type DeleteAuthorPayload {\n\tauthorId: ID!\n\teventSetId: ID!\n}"));
    }

    #[test]
    fn author_exposes_non_null_books_field() {
        let schema = build_schema(
            Query::new(MockQueryUseCase::new()),
            Mutation::new(MockMutationUseCase::new()),
        );

        assert!(schema.sdl().contains("\tbooks: [Book!]!"));
    }

    #[cfg(feature = "test-with-database")]
    #[sqlx::test]
    async fn authors_resolve_populated_shared_and_empty_book_lists(
        pool: sqlx::PgPool,
    ) -> anyhow::Result<()> {
        use async_graphql::dataloader::DataLoader;

        use crate::{
            dependency_injection::dependency_injection,
            presentation::graphql::loader::{AuthorLoader, BooksByAuthorLoader},
        };

        let author1 = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author2 = "93090e87-b7a1-403c-974c-d74d881e83b9";
        let author3 = "278935cf-ed83-4346-9b35-b84bbdb630c0";
        let book1 = "a1b2c3d4-e5f6-4890-abcd-ef1234567890";
        let book2 = "c5a81e57-bc91-40ff-8b57-18cfa7cc7ae8";
        sqlx::query("INSERT INTO bookshelf_user (id) VALUES ('user1')")
            .execute(&pool)
            .await?;
        sqlx::query(
            "INSERT INTO author (id, user_id, name) VALUES
             ($1, 'user1', 'Author 1'),
             ($2, 'user1', 'Author 2'),
             ($3, 'user1', 'Author 3')",
        )
        .bind(uuid::Uuid::parse_str(author1)?)
        .bind(uuid::Uuid::parse_str(author2)?)
        .bind(uuid::Uuid::parse_str(author3)?)
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO book
             (id, user_id, title, isbn, read, owned, priority, format, store)
             VALUES
             ($1, 'user1', 'Shared Book', '', false, true, 50, 'Unknown', 'Unknown'),
             ($2, 'user1', 'Author 1 Book', '', false, true, 50, 'Unknown', 'Unknown')",
        )
        .bind(uuid::Uuid::parse_str(book1)?)
        .bind(uuid::Uuid::parse_str(book2)?)
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO book_author (user_id, book_id, author_id) VALUES
             ('user1', $1, $2),
             ('user1', $1, $3),
             ('user1', $4, $2)",
        )
        .bind(uuid::Uuid::parse_str(book1)?)
        .bind(uuid::Uuid::parse_str(author1)?)
        .bind(uuid::Uuid::parse_str(author2)?)
        .bind(uuid::Uuid::parse_str(book2)?)
        .execute(&pool)
        .await?;

        let (query_use_case, schema) = dependency_injection(pool);
        let claims = Claims {
            sub: "user1".to_string(),
            _permissions: None,
        };
        let response = schema
            .execute(
                async_graphql::Request::from("query { authors { id books { id title } } }")
                    .data(claims.clone())
                    .data(DataLoader::new(
                        AuthorLoader::new(claims.clone(), query_use_case.clone()),
                        tokio::spawn,
                    ))
                    .data(DataLoader::new(
                        BooksByAuthorLoader::new(claims, query_use_case),
                        tokio::spawn,
                    )),
            )
            .await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        let json = serde_json::to_value(response)?;
        let authors = json["data"]["authors"].as_array().unwrap();
        let books_for = |author_id: &str| {
            authors
                .iter()
                .find(|author| author["id"] == author_id)
                .unwrap()["books"]
                .as_array()
                .unwrap()
        };

        assert_eq!(books_for(author1).len(), 2);
        assert_eq!(books_for(author2).len(), 1);
        assert!(books_for(author3).is_empty());

        Ok(())
    }
}
