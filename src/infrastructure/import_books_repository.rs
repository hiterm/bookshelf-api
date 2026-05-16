use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entity::{
    author::AuthorId,
    book::Book,
    event::{EventOperation, EventSetOperation},
    user::UserId,
};
use crate::domain::error::DomainError;
use crate::domain::repository::import_books_repository::{ImportBookInput, ImportBooksRepository};

#[derive(sqlx::FromRow)]
struct AuthorSnapshotRow {
    id: Uuid,
    yomi: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

/// NOTE: Temporary infrastructure repository. See `ImportBooksRepository` trait
/// documentation for rationale. Should be dissolved into aggregate
/// repositories once Unit of Work pattern is introduced.
#[derive(Debug, Clone)]
pub struct PgImportBooksRepository {
    pool: PgPool,
}

impl PgImportBooksRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ImportBooksRepository for PgImportBooksRepository {
    async fn import(
        &self,
        user_id: &UserId,
        books: Vec<ImportBookInput>,
    ) -> Result<Vec<Book>, DomainError> {
        let mut tx = self.pool.begin().await?;

        // Step 1 — generate the shared event_set ID.
        let es_id = Uuid::new_v4();
        sqlx::query("INSERT INTO event_set (id, user_id, operation) VALUES ($1, $2, $3)")
            .bind(es_id)
            .bind(user_id.as_str())
            .bind(EventSetOperation::ImportBooks.as_str())
            .execute(&mut *tx)
            .await?;

        // Step 2 — collect unique author names and build the name-to-ID map.
        let mut name_to_id: HashMap<String, AuthorId> = HashMap::new();

        for book in &books {
            for author_name in &book.author_names {
                let name = author_name.as_str().to_owned();
                if name_to_id.contains_key(&name) {
                    continue;
                }

                let candidate_id = Uuid::new_v4();

                let result = sqlx::query(
                    "INSERT INTO author (id, user_id, name) VALUES ($1, $2, $3)
                     ON CONFLICT (user_id, name) DO NOTHING",
                )
                .bind(candidate_id)
                .bind(user_id.as_str())
                .bind(&name)
                .execute(&mut *tx)
                .await?;

                let rows_affected = result.rows_affected();

                let snap: AuthorSnapshotRow = sqlx::query_as(
                    "SELECT id, yomi, created_at, updated_at
                     FROM author
                     WHERE user_id = $1 AND name = $2",
                )
                .bind(user_id.as_str())
                .bind(&name)
                .fetch_one(&mut *tx)
                .await?;

                let author_id = AuthorId::new(snap.id);
                name_to_id.insert(name.clone(), author_id.clone());

                if rows_affected == 1 {
                    sqlx::query(
                        "INSERT INTO author_event
                           (event_set_id, operation, author_id, user_id,
                            name, yomi, author_created_at, author_updated_at)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    )
                    .bind(EventOperation::Create.as_str())
                    .bind(es_id)
                    .bind(author_id.to_uuid())
                    .bind(user_id.as_str())
                    .bind(&name)
                    .bind(&snap.yomi)
                    .bind(snap.created_at)
                    .bind(snap.updated_at)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        // Step 3 — insert books and book events.
        let mut result_books = Vec::with_capacity(books.len());

        for book in books {
            let author_ids: Vec<AuthorId> = book
                .author_names
                .iter()
                .map(|name| {
                    name_to_id.get(name.as_str()).cloned().ok_or_else(|| {
                        DomainError::Unexpected(format!(
                            "author name '{}' not found in name_to_id map",
                            name.as_str()
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let book_entity = Book::new(
                book.book_id.clone(),
                book.title.clone(),
                author_ids.clone(),
                book.isbn.clone(),
                book.read.clone(),
                book.owned.clone(),
                book.priority.clone(),
                book.format,
                book.store,
                book.created_at,
                book.updated_at,
            )?;

            sqlx::query(
                "INSERT INTO book
                   (id, user_id, title, isbn, read, owned, priority, format, store,
                    created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            )
            .bind(book_entity.id().to_uuid())
            .bind(user_id.as_str())
            .bind(book_entity.title().as_str())
            .bind(book_entity.isbn().as_str())
            .bind(book_entity.read().to_bool())
            .bind(book_entity.owned().to_bool())
            .bind(book_entity.priority().to_i32())
            .bind(book_entity.format().to_string())
            .bind(book_entity.store().to_string())
            .bind(book_entity.created_at())
            .bind(book_entity.updated_at())
            .execute(&mut *tx)
            .await?;

            let author_uuids: Vec<Uuid> = author_ids.iter().map(|id| id.to_uuid()).collect();

            if !author_uuids.is_empty() {
                sqlx::query(
                    "INSERT INTO book_author (user_id, book_id, author_id)
                            SELECT $1, $2::uuid, * FROM UNNEST($3::uuid[])",
                )
                .bind(user_id.as_str())
                .bind(book_entity.id().to_uuid())
                .bind(&author_uuids)
                .execute(&mut *tx)
                .await?;
            }

            let (event_id,): (i64,) = sqlx::query_as(
                "INSERT INTO book_event
                   (event_set_id, operation, book_id, user_id,
                    title, isbn, read, owned, priority, format, store,
                    book_created_at, book_updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                 RETURNING event_id",
            )
            .bind(EventOperation::Create.as_str())
            .bind(es_id)
            .bind(book_entity.id().to_uuid())
            .bind(user_id.as_str())
            .bind(book_entity.title().as_str())
            .bind(book_entity.isbn().as_str())
            .bind(book_entity.read().to_bool())
            .bind(book_entity.owned().to_bool())
            .bind(book_entity.priority().to_i32())
            .bind(book_entity.format().to_string())
            .bind(book_entity.store().to_string())
            .bind(book_entity.created_at())
            .bind(book_entity.updated_at())
            .fetch_one(&mut *tx)
            .await?;

            if !author_uuids.is_empty() {
                sqlx::query(
                    "INSERT INTO book_event_author (event_id, author_id)
                            SELECT $1, * FROM UNNEST($2::uuid[])",
                )
                .bind(event_id)
                .bind(&author_uuids)
                .execute(&mut *tx)
                .await?;
            }

            result_books.push(book_entity);
        }

        // Step 4 — commit.
        tx.commit().await?;

        Ok(result_books)
    }
}

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {
    use time::{
        PrimitiveDateTime,
        macros::{date, time},
    };

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            entity::{
                author::{Author, AuthorName},
                book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                event::{EventOperation, EventSetOperation},
                user::User,
            },
            error::DomainError,
            repository::{
                author_repository::AuthorRepository, import_books_repository::ImportBookInput,
                user_repository::UserRepository,
            },
        },
        infrastructure::{
            author_repository::PgAuthorRepository, user_repository::PgUserRepository,
        },
    };

    use super::*;

    #[sqlx::test]
    async fn import_creates_new_authors_and_reuses_existing(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let import_repo = PgImportBooksRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;

        // Pre-create an existing author
        let existing_author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let existing_author = Author::new(
            existing_author_id,
            AuthorName::new("Existing Author".to_string())?,
        )?;
        author_repo.create(&user_id, &existing_author).await?;

        // Import two books: one with existing author, one with new author
        let inputs = vec![
            make_import_input("Book One", vec!["Existing Author"])?,
            make_import_input("Book Two", vec!["New Author"])?,
        ];

        let result = import_repo.import(&user_id, inputs).await?;
        assert_eq!(result.len(), 2);

        // Verify exactly 2 authors exist for user
        let author_rows: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM author WHERE user_id = $1 ORDER BY name")
                .bind(user_id.as_str())
                .fetch_all(&pool)
                .await?;
        assert_eq!(author_rows.len(), 2);
        assert_eq!(author_rows[0].0, "Existing Author");
        assert_eq!(author_rows[1].0, "New Author");

        // Verify exactly one author_event for the newly created author
        let query = format!(
            "SELECT COUNT(*) FROM author_event ae
             JOIN event_set es ON ae.event_set_id = es.id
             WHERE ae.user_id = $1 AND es.operation = '{}'",
            EventSetOperation::ImportBooks.as_str()
        );
        let (new_author_event_count,): (i64,) = sqlx::query_as(&query)
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(new_author_event_count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn import_deduplicates_shared_author_names(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let import_repo = PgImportBooksRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;

        // Import two books that share the same author
        let inputs = vec![
            make_import_input("Book One", vec!["Shared Author"])?,
            make_import_input("Book Two", vec!["Shared Author"])?,
        ];

        let result = import_repo.import(&user_id, inputs).await?;
        assert_eq!(result.len(), 2);

        // Only one author row should exist
        let (author_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(author_count, 1);

        // Each book should reference that single author
        let book_ids: Vec<(Uuid,)> =
            sqlx::query_as("SELECT book_id FROM book_author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_all(&pool)
                .await?;
        assert_eq!(book_ids.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn import_records_events_with_expected_fields(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let import_repo = PgImportBooksRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;

        let inputs = vec![make_import_input("Imported Book", vec!["Author A"])?];
        let result = import_repo.import(&user_id, inputs).await?;
        assert_eq!(result.len(), 1);

        // Verify event_set
        let query = format!(
            "SELECT operation FROM event_set WHERE user_id = $1 AND operation = '{}'",
            EventSetOperation::ImportBooks.as_str()
        );
        let (es_op,): (String,) = sqlx::query_as(&query)
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(es_op, EventSetOperation::ImportBooks.as_str());

        // Verify book_event
        let (be_op, be_title): (String, String) =
            sqlx::query_as("SELECT operation, title FROM book_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(be_op, EventOperation::Create.as_str());
        assert_eq!(be_title, "Imported Book");

        // Verify book_event_author
        let (book_event_author_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM book_event_author bea
             JOIN book_event be ON bea.event_id = be.event_id
             WHERE be.user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(book_event_author_count, 1);

        // Verify author_event
        let (ae_op, ae_name): (String, String) =
            sqlx::query_as("SELECT operation, name FROM author_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(ae_op, EventOperation::Create.as_str());
        assert_eq!(ae_name, "Author A");

        Ok(())
    }

    #[sqlx::test]
    async fn import_rolls_back_on_failure(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let import_repo = PgImportBooksRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;

        let shared_book_id = BookId::try_from("675bc8d9-3155-42fb-87b0-0a82cb162848")?;
        let now = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();

        // First book succeeds, second book has duplicate ID to force failure
        let inputs = vec![
            ImportBookInput {
                book_id: shared_book_id.clone(),
                title: BookTitle::new("First Book".to_owned())?,
                author_names: vec![AuthorName::new("Author A".to_owned())?],
                isbn: Isbn::new("".to_owned())?,
                read: ReadFlag::new(false),
                owned: OwnedFlag::new(false),
                priority: Priority::new(50)?,
                format: BookFormat::EBook,
                store: BookStore::Kindle,
                created_at: now,
                updated_at: now,
            },
            ImportBookInput {
                book_id: shared_book_id,
                title: BookTitle::new("Second Book".to_owned())?,
                author_names: vec![AuthorName::new("Author B".to_owned())?],
                isbn: Isbn::new("".to_owned())?,
                read: ReadFlag::new(false),
                owned: OwnedFlag::new(false),
                priority: Priority::new(50)?,
                format: BookFormat::EBook,
                store: BookStore::Kindle,
                created_at: now,
                updated_at: now,
            },
        ];

        let result = import_repo.import(&user_id, inputs).await;
        assert!(
            result.is_err(),
            "import should fail due to duplicate book_id"
        );

        // Assert no partial rows persisted
        let (book_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM book WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(book_count, 0, "no book rows should be persisted");

        let (author_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(author_count, 0, "no author rows should be persisted");

        let (event_set_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM event_set WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(event_set_count, 0, "no event_set rows should be persisted");

        let (book_event_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM book_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(
            book_event_count, 0,
            "no book_event rows should be persisted"
        );

        let (author_event_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(
            author_event_count, 0,
            "no author_event rows should be persisted"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn import_empty_author_names(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let import_repo = PgImportBooksRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;

        let inputs = vec![make_import_input("Book With No Authors", vec![])?];

        let result = import_repo.import(&user_id, inputs).await?;
        assert_eq!(result.len(), 1);

        // Verify book exists
        let (book_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM book WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(book_count, 1);

        // Verify no book_author rows created
        let (book_author_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM book_author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(
            book_author_count, 0,
            "book_author should be empty when no authors"
        );

        // Verify no book_event_author rows created
        let (book_event_author_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM book_event_author bea
             JOIN book_event be ON bea.event_id = be.event_id
             WHERE be.user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(
            book_event_author_count, 0,
            "book_event_author should be empty when no authors"
        );

        Ok(())
    }

    async fn prepare_user(repository: &PgUserRepository, id: &str) -> Result<UserId, DomainError> {
        let user_id = UserId::new(String::from(id))?;
        let user = User::new(user_id.clone());
        repository.create(&user).await?;
        Ok(user_id)
    }

    fn make_import_input(
        title: &str,
        author_names: Vec<&str>,
    ) -> Result<ImportBookInput, DomainError> {
        let now = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        Ok(ImportBookInput {
            book_id: BookId::new(Uuid::new_v4())?,
            title: BookTitle::new(title.to_owned())?,
            author_names: author_names
                .into_iter()
                .map(|n| AuthorName::new(n.to_owned()))
                .collect::<Result<Vec<_>, _>>()?,
            isbn: Isbn::new("".to_owned())?,
            read: ReadFlag::new(false),
            owned: OwnedFlag::new(false),
            priority: Priority::new(50)?,
            format: BookFormat::EBook,
            store: BookStore::Kindle,
            created_at: now,
            updated_at: now,
        })
    }
}
