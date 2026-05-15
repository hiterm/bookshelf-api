use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entity::{author::AuthorId, book::Book, user::UserId};
use crate::domain::error::DomainError;
use crate::domain::repository::import_books_repository::{ImportBookInput, ImportBooksRepository};

#[derive(sqlx::FromRow)]
struct AuthorSnapshotRow {
    id: Uuid,
    yomi: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

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
        sqlx::query(
            "INSERT INTO event_set (id, user_id, operation) VALUES ($1, $2, 'import_books')",
        )
        .bind(es_id)
        .bind(user_id.as_str())
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
                         VALUES ($1, 'create', $2, $3, $4, $5, $6, $7)",
                    )
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
                .map(|name| name_to_id[name.as_str()].clone())
                .collect();

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
                 VALUES ($1, 'create', $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                 RETURNING event_id",
            )
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
