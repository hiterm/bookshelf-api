use crate::use_case::traits::{
    author::{AuthorCommandUseCase, AuthorQueryUseCase},
    book::{BookCommandUseCase, BookQueryUseCase},
    history::{HistoryCommandUseCase, HistoryQueryUseCase},
    user::{UserCommandUseCase, UserQueryUseCase},
};

use super::{mutation::Mutation, query::Query};
use async_graphql::{EmptySubscription, Schema};

pub type GraphqlSchema<UQ, BQ, AQ, HQ, UC, BC, AC, HC> =
    Schema<Query<UQ, BQ, AQ, HQ>, Mutation<UC, BC, AC, HC>, EmptySubscription>;

pub fn build_schema<UQ, BQ, AQ, HQ, UC, BC, AC, HC>(
    query: Query<UQ, BQ, AQ, HQ>,
    mutation: Mutation<UC, BC, AC, HC>,
) -> GraphqlSchema<UQ, BQ, AQ, HQ, UC, BC, AC, HC>
where
    UQ: UserQueryUseCase,
    BQ: BookQueryUseCase,
    AQ: AuthorQueryUseCase,
    HQ: HistoryQueryUseCase,
    UC: UserCommandUseCase,
    BC: BookCommandUseCase,
    AC: AuthorCommandUseCase,
    HC: HistoryCommandUseCase,
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
            dto::{
                author::AuthorDto,
                history::{AuthorRevisionDto, BookRevisionDto, OperationDto},
                mutation::SingleRevisionMutationResultDto,
            },
            traits::{
                author::{MockAuthorCommandUseCase, MockAuthorQueryUseCase},
                book::{MockBookCommandUseCase, MockBookQueryUseCase},
                history::{MockHistoryCommandUseCase, MockHistoryQueryUseCase},
                user::{MockUserCommandUseCase, MockUserQueryUseCase},
            },
        },
    };

    use super::build_schema;

    fn query() -> Query<
        MockUserQueryUseCase,
        MockBookQueryUseCase,
        MockAuthorQueryUseCase,
        MockHistoryQueryUseCase,
    > {
        Query::new(
            MockUserQueryUseCase::new(),
            MockBookQueryUseCase::new(),
            MockAuthorQueryUseCase::new(),
            MockHistoryQueryUseCase::new(),
        )
    }

    fn mutation() -> Mutation<
        MockUserCommandUseCase,
        MockBookCommandUseCase,
        MockAuthorCommandUseCase,
        MockHistoryCommandUseCase,
    > {
        Mutation::new(
            MockUserCommandUseCase::new(),
            MockBookCommandUseCase::new(),
            MockAuthorCommandUseCase::new(),
            MockHistoryCommandUseCase::new(),
        )
    }

    #[tokio::test]
    async fn execute_query() {
        let user_id = "user1";
        let author_id = "d065a358-4fa7-4236-ae19-f6f2f9467c35";
        let author_name = "author1";

        let mut author_query = MockAuthorQueryUseCase::new();
        author_query
            .expect_find_by_id()
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
        let query = Query::new(
            MockUserQueryUseCase::new(),
            MockBookQueryUseCase::new(),
            author_query,
            MockHistoryQueryUseCase::new(),
        );
        let mutation = mutation();
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
    async fn create_author_payload_exposes_operation_and_revision() {
        let mut author_command = MockAuthorCommandUseCase::new();
        author_command
            .expect_create()
            .with(predicate::eq("user1"), predicate::always())
            .returning(|_, input| {
                Ok(SingleRevisionMutationResultDto::new(
                    AuthorDto {
                        id: "d065a358-4fa7-4236-ae19-f6f2f9467c35".to_string(),
                        name: input.name,
                        yomi: String::new(),
                        created_at: time::OffsetDateTime::UNIX_EPOCH,
                        updated_at: time::OffsetDateTime::UNIX_EPOCH,
                    },
                    "e77df9d5-b7bf-47f2-8753-03f285d440e3".to_string(),
                    1234,
                ))
            });
        let schema = build_schema(
            query(),
            Mutation::new(
                MockUserCommandUseCase::new(),
                MockBookCommandUseCase::new(),
                author_command,
                MockHistoryCommandUseCase::new(),
            ),
        );
        let claims = Claims {
            sub: "user1".to_string(),
            _permissions: None,
        };

        let response = schema
            .execute(
                async_graphql::Request::from(
                    r#"mutation { createAuthor(authorData: { name: "Author" }) { operationId revisionNumber author { name } } }"#,
                )
                .data(claims),
            )
            .await;
        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json["data"]["createAuthor"]["revisionNumber"], 1234);
        assert_eq!(
            json["data"]["createAuthor"]["operationId"],
            "e77df9d5-b7bf-47f2-8753-03f285d440e3"
        );
        assert_eq!(json["data"]["createAuthor"]["author"]["name"], "Author");
    }

    #[tokio::test]
    async fn undo_operation_returns_the_new_operation_id() {
        let target_id = "e77df9d5-b7bf-47f2-8753-03f285d440e3";
        let undo_id = "79455a41-bb67-44a7-966f-2f6fdd04e8ca";
        let mut history_command = MockHistoryCommandUseCase::new();
        history_command
            .expect_undo_operation()
            .with(predicate::eq("user1"), predicate::eq(target_id))
            .returning(move |_, _| Ok(undo_id.to_owned()));
        let schema = build_schema(
            query(),
            Mutation::new(
                MockUserCommandUseCase::new(),
                MockBookCommandUseCase::new(),
                MockAuthorCommandUseCase::new(),
                history_command,
            ),
        );
        let response = schema
            .execute(
                async_graphql::Request::from(format!(
                    "mutation {{ undoOperation(operationId: \"{target_id}\") {{ operationId }} }}"
                ))
                .data(Claims {
                    sub: "user1".to_owned(),
                    _permissions: None,
                }),
            )
            .await;
        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json["data"]["undoOperation"]["operationId"], undo_id);
    }

    #[tokio::test]
    async fn history_resolvers_forward_owner_ids_and_revision_numbers() {
        use crate::common::types::{BookFormat, BookStore};

        let operation_id = "e77df9d5-b7bf-47f2-8753-03f285d440e3";
        let book_id = "d065a358-4fa7-4236-ae19-f6f2f9467c35";
        let author_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let operation = OperationDto {
            id: operation_id.to_string(),
            operation_type: "create_book".to_string(),
            detail: None,
            undo_of_operation_id: None,
            created_at: time::OffsetDateTime::UNIX_EPOCH,
        };
        let book_revision = BookRevisionDto {
            book_id: book_id.to_string(),
            revision_number: 2,
            title: "Revision Title".to_string(),
            author_ids: vec![author_id.to_string()],
            isbn: String::new(),
            read: false,
            owned: true,
            priority: 50,
            format: BookFormat::Printed,
            store: BookStore::Unknown,
            purchase_date: None,
            book_created_at: time::OffsetDateTime::UNIX_EPOCH,
            book_updated_at: time::OffsetDateTime::UNIX_EPOCH,
            created_at: time::OffsetDateTime::UNIX_EPOCH,
        };
        let author_revision = AuthorRevisionDto {
            author_id: author_id.to_string(),
            revision_number: 3,
            name: "Revision Author".to_string(),
            yomi: String::new(),
            author_created_at: time::OffsetDateTime::UNIX_EPOCH,
            author_updated_at: time::OffsetDateTime::UNIX_EPOCH,
            created_at: time::OffsetDateTime::UNIX_EPOCH,
        };
        let mut history = MockHistoryQueryUseCase::new();
        let listed_operation = operation.clone();
        history
            .expect_operations()
            .with(predicate::eq("user1"))
            .return_once(move |_| Ok(vec![listed_operation]));
        history
            .expect_operation()
            .with(predicate::eq("user1"), predicate::eq(operation_id))
            .return_once(move |_, _| Ok(Some(operation)));
        let listed_book_revision = book_revision.clone();
        history
            .expect_book_revisions()
            .with(predicate::eq("user1"), predicate::eq(book_id))
            .return_once(move |_, _| Ok(vec![listed_book_revision]));
        history
            .expect_book_revision()
            .with(
                predicate::eq("user1"),
                predicate::eq(book_id),
                predicate::eq(2),
            )
            .return_once(move |_, _, _| Ok(Some(book_revision)));
        let listed_author_revision = author_revision.clone();
        history
            .expect_author_revisions()
            .with(predicate::eq("user1"), predicate::eq(author_id))
            .return_once(move |_, _| Ok(vec![listed_author_revision]));
        history
            .expect_author_revision()
            .with(
                predicate::eq("user1"),
                predicate::eq(author_id),
                predicate::eq(3),
            )
            .return_once(move |_, _, _| Ok(Some(author_revision)));
        let schema = build_schema(
            Query::new(
                MockUserQueryUseCase::new(),
                MockBookQueryUseCase::new(),
                MockAuthorQueryUseCase::new(),
                history,
            ),
            mutation(),
        );

        let response = schema
            .execute(
                async_graphql::Request::from(format!(
                    r#"{{
                        operations {{ id type }}
                        operation(id: "{operation_id}") {{ id type }}
                        bookRevisions(bookId: "{book_id}") {{ revisionNumber title }}
                        bookRevision(bookId: "{book_id}", revisionNumber: 2) {{ revisionNumber title }}
                        authorRevisions(authorId: "{author_id}") {{ revisionNumber name }}
                        authorRevision(authorId: "{author_id}", revisionNumber: 3) {{ revisionNumber name }}
                    }}"#
                ))
                .data(Claims {
                    sub: "user1".to_string(),
                    _permissions: None,
                }),
            )
            .await;
        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json["data"]["operations"][0]["type"], "create_book");
        assert_eq!(json["data"]["operation"]["id"], operation_id);
        assert_eq!(json["data"]["bookRevisions"][0]["title"], "Revision Title");
        assert_eq!(json["data"]["bookRevision"]["revisionNumber"], 2);
        assert_eq!(
            json["data"]["authorRevisions"][0]["name"],
            "Revision Author"
        );
        assert_eq!(json["data"]["authorRevision"]["revisionNumber"], 3);
    }

    #[test]
    fn mutation_payloads_expose_only_canonical_fields() {
        let query = query();
        let mutation = mutation();
        let sdl = build_schema(query, mutation).sdl();

        assert!(sdl.contains(
            "type BookMutationPayload {\n\tbook: Book!\n\toperationId: ID!\n\trevisionNumber: Int!\n}"
        ));
        assert!(sdl.contains(
            "type AuthorMutationPayload {\n\tauthor: Author!\n\toperationId: ID!\n\trevisionNumber: Int!\n}"
        ));
        assert!(sdl.contains("type DeleteBookPayload {\n\tbookId: ID!\n\toperationId: ID!\n}"));
        assert!(sdl.contains("type DeleteAuthorPayload {\n\tauthorId: ID!\n\toperationId: ID!\n}"));
    }

    #[test]
    fn author_exposes_non_null_books_field() {
        let schema = build_schema(query(), mutation());

        assert!(schema.sdl().contains("\tbooks: [Book!]!"));
        assert!(schema.sdl().matches("purchaseDate: Date").count() >= 6);
    }

    #[test]
    fn legacy_event_graphql_contract_is_absent() {
        let sdl = build_schema(query(), mutation()).sdl();

        for legacy_name in [
            "type EventSet",
            "type BookEventEntry",
            "type AuthorEventEntry",
            "\teventSets:",
            "\teventSet(",
            "\tbookEvents(",
            "\tauthorEvents(",
        ] {
            assert!(!sdl.contains(legacy_name), "found {legacy_name} in SDL");
        }
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

        let (author_query, book_query, _history_query, schema) = dependency_injection(pool);
        let claims = Claims {
            sub: "user1".to_string(),
            _permissions: None,
        };
        let response = schema
            .execute(
                async_graphql::Request::from("query { authors { id books { id title } } }")
                    .data(claims.clone())
                    .data(DataLoader::new(
                        AuthorLoader::new(claims.clone(), author_query),
                        tokio::spawn,
                    ))
                    .data(DataLoader::new(
                        BooksByAuthorLoader::new(claims, book_query),
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
