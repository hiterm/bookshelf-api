use std::collections::HashMap;

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        author::{Author, AuthorId, AuthorName},
        event::EventOperation,
        user::UserId,
    },
    error::DomainError,
    repository::author_repository::AuthorRepository,
};
use crate::infrastructure::transaction::PgTransaction;

#[derive(sqlx::FromRow)]
struct AuthorRow {
    id: Uuid,
    name: String,
    yomi: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct AuthorSnapshotRow {
    name: String,
    yomi: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

// Used by find_or_create_by_name to read the DB-generated id and timestamps
// after an ON CONFLICT DO NOTHING insert.
#[derive(sqlx::FromRow)]
struct AuthorIdSnapshotRow {
    id: Uuid,
    yomi: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct PgAuthorRepository {
    pool: PgPool,
}

impl PgAuthorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorRepository for PgAuthorRepository {
    type Transaction = PgTransaction;

    async fn create(&self, tx: &mut Self::Transaction, author: &Author) -> Result<(), DomainError> {
        let user_id = tx.user_id().clone();
        sqlx::query(
            "INSERT INTO author (id, user_id, name, yomi, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .bind(author.name().as_str())
        .bind(author.yomi())
        .bind(author.created_at())
        .bind(author.updated_at())
        .execute(tx.as_mut())
        .await?;

        // Fetch the just-inserted row for the event snapshot.
        let snap: AuthorSnapshotRow = sqlx::query_as(
            "SELECT name, yomi, created_at, updated_at FROM author WHERE id = $1 AND user_id = $2",
        )
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .fetch_one(tx.as_mut())
        .await?;

        sqlx::query(
            "INSERT INTO author_event
               (event_set_id, operation, author_id, user_id, name, yomi,
                author_created_at, author_updated_at)
             VALUES ($1, 'create', $2, $3, $4, $5, $6, $7)",
        )
        .bind(tx.event_set_id())
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .bind(&snap.name)
        .bind(&snap.yomi)
        .bind(snap.created_at)
        .bind(snap.updated_at)
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }

    async fn find_or_create_by_name(
        &self,
        tx: &mut Self::Transaction,
        name: &AuthorName,
        created_at: OffsetDateTime,
    ) -> Result<AuthorId, DomainError> {
        let user_id = tx.user_id().clone();
        let name = name.as_str();
        let candidate_id = Uuid::new_v4();

        let result = sqlx::query(
            "INSERT INTO author (id, user_id, name, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $4)
             ON CONFLICT (user_id, name) DO NOTHING",
        )
        .bind(candidate_id)
        .bind(user_id.as_str())
        .bind(name)
        .bind(created_at)
        .execute(tx.as_mut())
        .await?;

        let rows_affected = result.rows_affected();

        let snap: AuthorIdSnapshotRow = sqlx::query_as(
            "SELECT id, yomi, created_at, updated_at
             FROM author
             WHERE user_id = $1 AND name = $2",
        )
        .bind(user_id.as_str())
        .bind(name)
        .fetch_one(tx.as_mut())
        .await?;

        let author_id = AuthorId::new(snap.id);

        if rows_affected == 1 {
            sqlx::query(
                "INSERT INTO author_event
                   (event_set_id, operation, author_id, user_id,
                    name, yomi, author_created_at, author_updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(tx.event_set_id())
            .bind(EventOperation::Create.as_str())
            .bind(author_id.to_uuid())
            .bind(user_id.as_str())
            .bind(name)
            .bind(&snap.yomi)
            .bind(snap.created_at)
            .bind(snap.updated_at)
            .execute(tx.as_mut())
            .await?;
        }

        Ok(author_id)
    }

    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError> {
        find_author_by_id_with_executor(&self.pool, user_id, author_id).await
    }

    async fn find_by_id_with_tx(
        &self,
        tx: &mut Self::Transaction,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError> {
        let row: Option<AuthorRow> =
            sqlx::query_as("SELECT * FROM author WHERE id = $1 AND user_id = $2 FOR UPDATE")
                .bind(author_id.to_uuid())
                .bind(user_id.as_str())
                .fetch_optional(tx.as_mut())
                .await?;

        author_from_optional_row(row)
    }

    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Author>, DomainError> {
        let authors: Result<Vec<Author>, DomainError> =
            sqlx::query_as("SELECT * FROM author WHERE user_id = $1 ORDER BY name ASC")
                .bind(user_id.as_str())
                .fetch(&self.pool)
                .map(
                    |row: Result<AuthorRow, sqlx::Error>| -> Result<Author, DomainError> {
                        let row = row?;
                        let author_id = AuthorId::new(row.id);
                        let author_name = AuthorName::new(row.name)?;
                        let author = Author::new_with_timestamps(
                            author_id,
                            author_name,
                            row.yomi,
                            row.created_at,
                            row.updated_at,
                        )?;
                        Ok(author)
                    },
                )
                .try_collect()
                .await;

        authors
    }

    async fn update(&self, tx: &mut Self::Transaction, author: &Author) -> Result<(), DomainError> {
        let user_id = tx.user_id().clone();
        let result = sqlx::query(
            "UPDATE author SET name = $1, yomi = $2, updated_at = $3
             WHERE id = $4 AND user_id = $5",
        )
        .bind(author.name().as_str())
        .bind(author.yomi())
        .bind(author.updated_at())
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .execute(tx.as_mut())
        .await?;

        match result.rows_affected() {
            0 => {
                return Err(DomainError::NotFound {
                    entity_type: "author",
                    entity_id: author.id().to_string(),
                    user_id: user_id.to_owned().into_string(),
                });
            }
            1 => {}
            _ => {
                return Err(DomainError::Unexpected(String::from(
                    "rows_affected is greater than 1.",
                )));
            }
        }

        // Fetch post-update state and DB-managed timestamps.
        let snap: AuthorSnapshotRow = sqlx::query_as(
            "SELECT name, yomi, created_at, updated_at FROM author WHERE id = $1 AND user_id = $2",
        )
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .fetch_one(tx.as_mut())
        .await?;

        sqlx::query(
            "INSERT INTO author_event
               (event_set_id, operation, author_id, user_id, name, yomi,
                author_created_at, author_updated_at)
             VALUES ($1, 'update', $2, $3, $4, $5, $6, $7)",
        )
        .bind(tx.event_set_id())
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .bind(&snap.name)
        .bind(&snap.yomi)
        .bind(snap.created_at)
        .bind(snap.updated_at)
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }

    async fn delete(
        &self,
        tx: &mut Self::Transaction,
        author_id: &AuthorId,
    ) -> Result<(), DomainError> {
        let user_id = tx.user_id().clone();
        // Lock the author row to prevent concurrent inserts into book_author after the count check.
        let exists: Option<(i32,)> =
            sqlx::query_as("SELECT 1 FROM author WHERE id = $1 AND user_id = $2 FOR UPDATE")
                .bind(author_id.to_uuid())
                .bind(user_id.as_str())
                .fetch_optional(tx.as_mut())
                .await?;

        if exists.is_none() {
            return Err(DomainError::NotFound {
                entity_type: "author",
                entity_id: author_id.to_string(),
                user_id: user_id.to_owned().into_string(),
            });
        }

        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM book_author WHERE user_id = $1 AND author_id = $2",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .fetch_one(tx.as_mut())
        .await?;

        if count > 0 {
            return Err(DomainError::HasAssociatedBooks {
                author_id: author_id.to_string(),
                user_id: user_id.to_owned().into_string(),
            });
        }

        let result = sqlx::query("DELETE FROM author WHERE id = $1 AND user_id = $2")
            .bind(author_id.to_uuid())
            .bind(user_id.as_str())
            .execute(tx.as_mut())
            .await?;

        match result.rows_affected() {
            0 => {
                return Err(DomainError::NotFound {
                    entity_type: "author",
                    entity_id: author_id.to_string(),
                    user_id: user_id.to_owned().into_string(),
                });
            }
            1 => {}
            _ => {
                return Err(DomainError::Unexpected(String::from(
                    "rows_affected is greater than 1.",
                )));
            }
        }

        sqlx::query(
            "INSERT INTO author_event (event_set_id, operation, author_id, user_id)
             VALUES ($1, 'delete', $2, $3)",
        )
        .bind(tx.event_set_id())
        .bind(author_id.to_uuid())
        .bind(user_id.as_str())
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }

    async fn restore(
        &self,
        tx: &mut Self::Transaction,
        source_event_id: i64,
        author: Option<Author>,
    ) -> Result<(), DomainError> {
        let user_id = tx.user_id().clone();
        let extra = json!({"version": 1, "source_event_id": source_event_id});

        match author {
            Some(author) => {
                let result = sqlx::query(
                    "UPDATE author SET name=$2, yomi=$3, created_at=$4, updated_at=$5
                         WHERE id=$1 AND user_id=$6",
                )
                .bind(author.id().to_uuid())
                .bind(author.name().as_str())
                .bind(author.yomi())
                .bind(author.created_at())
                .bind(author.updated_at())
                .bind(user_id.as_str())
                .execute(tx.as_mut())
                .await?;

                if result.rows_affected() == 0 {
                    sqlx::query(
                        "INSERT INTO author
                           (id, user_id, name, yomi, created_at, updated_at)
                         VALUES ($1, $2, $3, $4, $5, $6)",
                    )
                    .bind(author.id().to_uuid())
                    .bind(user_id.as_str())
                    .bind(author.name().as_str())
                    .bind(author.yomi())
                    .bind(author.created_at())
                    .bind(author.updated_at())
                    .execute(tx.as_mut())
                    .await?;
                }

                let snapshot: AuthorSnapshotRow = sqlx::query_as(
                    "SELECT name, yomi, created_at, updated_at FROM author
                     WHERE id = $1 AND user_id = $2",
                )
                .bind(author.id().to_uuid())
                .bind(user_id.as_str())
                .fetch_one(tx.as_mut())
                .await?;

                sqlx::query(
                    "INSERT INTO author_event
                       (event_set_id, operation, author_id, user_id,
                        name, yomi, author_created_at, author_updated_at, extra)
                     VALUES ($1, 'restore', $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(tx.event_set_id())
                .bind(author.id().to_uuid())
                .bind(user_id.as_str())
                .bind(&snapshot.name)
                .bind(&snapshot.yomi)
                .bind(snapshot.created_at)
                .bind(snapshot.updated_at)
                .bind(sqlx::types::Json(&extra))
                .execute(tx.as_mut())
                .await?;
            }
            None => {
                let (author_id,): (Uuid,) = sqlx::query_as(
                    "SELECT author_id FROM author_event WHERE event_id = $1 AND user_id = $2",
                )
                .bind(source_event_id)
                .bind(user_id.as_str())
                .fetch_one(tx.as_mut())
                .await?;

                // 0 rows affected is acceptable (author already absent)
                sqlx::query("DELETE FROM author WHERE id=$1 AND user_id=$2")
                    .bind(author_id)
                    .bind(user_id.as_str())
                    .execute(tx.as_mut())
                    .await?;

                sqlx::query(
                    "INSERT INTO author_event (event_set_id, operation, author_id, user_id, extra)
                     VALUES ($1, 'restore', $2, $3, $4)",
                )
                .bind(tx.event_set_id())
                .bind(author_id)
                .bind(user_id.as_str())
                .bind(sqlx::types::Json(&extra))
                .execute(tx.as_mut())
                .await?;
            }
        }

        Ok(())
    }

    async fn find_by_ids_as_hash_map(
        &self,
        user_id: &UserId,
        author_ids: &[AuthorId],
    ) -> Result<HashMap<AuthorId, Author>, DomainError> {
        let author_ids: Vec<Uuid> = author_ids
            .iter()
            .map(|author_id| author_id.to_uuid())
            .collect();

        let authors_map: HashMap<AuthorId, Author> = sqlx::query_as(
            "SELECT * FROM author WHERE user_id = $1 AND id = ANY($2) ORDER BY name ASC",
        )
        .bind(user_id.as_str())
        .bind(author_ids)
        .fetch(&self.pool)
        .map(
            |row: Result<AuthorRow, sqlx::Error>| -> Result<(AuthorId, Author), DomainError> {
                let row = row?;
                let author_id = AuthorId::new(row.id);
                let author_name = AuthorName::new(row.name)?;
                let author = Author::new_with_timestamps(
                    author_id.clone(),
                    author_name,
                    row.yomi,
                    row.created_at,
                    row.updated_at,
                )?;
                Ok((author_id, author))
            },
        )
        .try_collect()
        .await?;

        Ok(authors_map)
    }
}

async fn find_author_by_id_with_executor<'e, E>(
    executor: E,
    user_id: &UserId,
    author_id: &AuthorId,
) -> Result<Option<Author>, DomainError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let row: Option<AuthorRow> =
        sqlx::query_as("SELECT * FROM author WHERE id = $1 AND user_id = $2")
            .bind(author_id.to_uuid())
            .bind(user_id.as_str())
            .fetch_optional(executor)
            .await?;

    author_from_optional_row(row)
}

fn author_from_optional_row(row: Option<AuthorRow>) -> Result<Option<Author>, DomainError> {
    row.map(|row| -> Result<Author, DomainError> {
        let author_id: AuthorId = row.id.into();
        let author_name = AuthorName::new(row.name)?;
        Author::new_with_timestamps(
            author_id,
            author_name,
            row.yomi,
            row.created_at,
            row.updated_at,
        )
    })
    .transpose()
}

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            entity::{
                book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                event::EventSetOperation,
                user::User,
            },
            error::DomainError,
            repository::{
                book_repository::BookRepository, transaction::TransactionManager,
                user_repository::UserRepository,
            },
        },
        infrastructure::{
            book_repository::PgBookRepository, transaction::PgTransactionManager,
            user_repository::PgUserRepository,
        },
    };
    use time::{
        PrimitiveDateTime,
        macros::{date, time},
    };

    use super::*;

    // Wrap a BookRepository::create in a single transaction opened via
    // PgTransactionManager, used to set up books in author tests.
    async fn create_book(
        pool: &PgPool,
        book_repository: &PgBookRepository,
        user_id: &UserId,
        book: &Book,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::CreateBook).await?;
        book_repository.create(&mut tx, book).await?;
        tm.commit(tx).await
    }

    // Wrap each AuthorRepository mutation in a single transaction opened via
    // PgTransactionManager, mirroring how the use-case layer drives it.
    async fn create_author(
        pool: &PgPool,
        author_repository: &PgAuthorRepository,
        user_id: &UserId,
        author: &Author,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::CreateAuthor).await?;
        author_repository.create(&mut tx, author).await?;
        tm.commit(tx).await
    }

    async fn update_author(
        pool: &PgPool,
        author_repository: &PgAuthorRepository,
        user_id: &UserId,
        author: &Author,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::UpdateAuthor).await?;
        author_repository.update(&mut tx, author).await?;
        tm.commit(tx).await
    }

    async fn delete_author(
        pool: &PgPool,
        author_repository: &PgAuthorRepository,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::DeleteAuthor).await?;
        author_repository.delete(&mut tx, author_id).await?;
        tm.commit(tx).await
    }

    async fn restore_author(
        pool: &PgPool,
        author_repository: &PgAuthorRepository,
        user_id: &UserId,
        source_event_id: i64,
        author: Author,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::RestoreAuthor).await?;
        author_repository
            .restore(&mut tx, source_event_id, Some(author))
            .await?;
        tm.commit(tx).await
    }

    fn new_author(id: AuthorId, name: AuthorName) -> Result<Author, DomainError> {
        Author::new(id, name, OffsetDateTime::UNIX_EPOCH)
    }

    #[sqlx::test]
    async fn create_and_find_by_id(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author = new_author(author_id.clone(), author_name)?;

        create_author(&pool, &author_repository, &user_id, &author).await?;

        let actual = author_repository.find_by_id(&user_id, &author_id).await?;
        assert_eq!(actual, Some(author.clone()));

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_id_with_tx_matches_find_by_id(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new(String::from("author1"))?)?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        let expected = author_repository.find_by_id(&user_id, &author_id).await?;

        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(&user_id, EventSetOperation::UpdateAuthor).await?;
        let actual = author_repository
            .find_by_id_with_tx(&mut tx, &user_id, &author_id)
            .await?;
        assert_eq!(actual, expected);
        tm.commit(tx).await?;

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_id_with_tx_uses_explicit_user_scope(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;
        let other_user_id = UserId::new(String::from("user2"))?;
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new(String::from("author1"))?)?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(&user_id, EventSetOperation::UpdateAuthor).await?;
        let actual = author_repository
            .find_by_id_with_tx(&mut tx, &other_user_id, &author_id)
            .await?;
        assert_eq!(actual, None);
        tm.commit(tx).await?;

        Ok(())
    }

    #[sqlx::test]
    async fn create_and_find_all(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author1 = new_author(author_id.clone(), author_name)?;

        let author_id = AuthorId::try_from("e9700384-6217-4152-88c0-7ba38aeee73a")?;
        let author_name = AuthorName::new(String::from("author2"))?;
        let author2 = new_author(author_id.clone(), author_name)?;

        create_author(&pool, &author_repository, &user_id, &author1).await?;
        create_author(&pool, &author_repository, &user_id, &author2).await?;

        let all_authors = author_repository.find_all(&user_id).await?;
        assert_eq!(all_authors.len(), 2);
        assert_eq!(all_authors, vec![author1, author2]);

        Ok(())
    }

    #[sqlx::test]
    async fn create_and_find_by_ids_as_hash_map(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id1 = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author1 = new_author(author_id1.clone(), author_name)?;

        let author_id2 = AuthorId::try_from("e9700384-6217-4152-88c0-7ba38aeee73a")?;
        let author_name = AuthorName::new(String::from("author2"))?;
        let author2 = new_author(author_id2.clone(), author_name)?;

        create_author(&pool, &author_repository, &user_id, &author1).await?;
        create_author(&pool, &author_repository, &user_id, &author2).await?;

        let all_authors = author_repository
            .find_by_ids_as_hash_map(&user_id, &[author_id1.clone(), author_id2.clone()])
            .await?;
        let mut expected = HashMap::new();
        expected.insert(author_id1, author1);
        expected.insert(author_id2, author2);

        assert_eq!(all_authors.len(), 2);
        assert_eq!(all_authors, expected);

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_id_does_not_return_other_users_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        create_author(&pool, &author_repository, &user1_id, &author).await?;

        let result = author_repository.find_by_id(&user2_id, &author_id).await?;
        assert_eq!(result, None);

        Ok(())
    }

    #[sqlx::test]
    async fn find_all_does_not_return_other_users_authors(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id, AuthorName::new("author1".to_string())?)?;
        create_author(&pool, &author_repository, &user1_id, &author).await?;

        let result = author_repository.find_all(&user2_id).await?;
        assert_eq!(result.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_ids_as_hash_map_does_not_return_other_users_authors(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        create_author(&pool, &author_repository, &user1_id, &author).await?;

        let result = author_repository
            .find_by_ids_as_hash_map(&user2_id, &[author_id])
            .await?;
        assert_eq!(result.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn update_changes_name(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("original".to_string())?)?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        let updated = new_author(author_id.clone(), AuthorName::new("updated".to_string())?)?;
        update_author(&pool, &author_repository, &user_id, &updated).await?;

        let actual = author_repository.find_by_id(&user_id, &author_id).await?;
        assert_eq!(
            actual.map(|a| a.name().as_str().to_string()),
            Some("updated".to_string())
        );

        Ok(())
    }

    #[sqlx::test]
    async fn update_returns_not_found_for_nonexistent_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id, AuthorName::new("name".to_string())?)?;

        let result = update_author(&pool, &author_repository, &user_id, &author).await;
        assert!(matches!(result, Err(DomainError::NotFound { .. })));

        Ok(())
    }

    #[sqlx::test]
    async fn update_returns_not_found_for_other_users_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("name".to_string())?)?;
        create_author(&pool, &author_repository, &user1_id, &author).await?;

        let updated = new_author(author_id, AuthorName::new("hacked".to_string())?)?;
        let result = update_author(&pool, &author_repository, &user2_id, &updated).await;
        assert!(matches!(result, Err(DomainError::NotFound { .. })));

        Ok(())
    }

    #[sqlx::test]
    async fn delete_fails_when_author_has_associated_books(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        let book = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            std::slice::from_ref(&author_id),
        )?;
        create_book(&pool, &book_repository, &user_id, &book).await?;

        let result = delete_author(&pool, &author_repository, &user_id, &author_id).await;
        assert!(matches!(
            result,
            Err(DomainError::HasAssociatedBooks { .. })
        ));

        // author and book_author must still exist
        let found = author_repository.find_by_id(&user_id, &author_id).await?;
        assert!(found.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_succeeds_when_author_has_no_associated_books(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        delete_author(&pool, &author_repository, &user_id, &author_id).await?;

        let found = author_repository.find_by_id(&user_id, &author_id).await?;
        assert_eq!(found, None);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_returns_not_found_for_nonexistent_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let result = delete_author(&pool, &author_repository, &user_id, &author_id).await;
        assert!(matches!(result, Err(DomainError::NotFound { .. })));

        Ok(())
    }

    #[sqlx::test]
    async fn delete_does_not_affect_other_users_when_book_association_blocks(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        // Both users have the same author UUID — allowed by composite PK (id, user_id)
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author1 = new_author(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        let author2 = new_author(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        create_author(&pool, &author_repository, &user1_id, &author1).await?;
        create_author(&pool, &author_repository, &user2_id, &author2).await?;

        let book1 = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            std::slice::from_ref(&author_id),
        )?;
        create_book(&pool, &book_repository, &user1_id, &book1).await?;
        let book2 = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            std::slice::from_ref(&author_id),
        )?;
        create_book(&pool, &book_repository, &user2_id, &book2).await?;

        // user2 has an associated book, so delete must fail
        let result = delete_author(&pool, &author_repository, &user2_id, &author_id).await;
        assert!(matches!(
            result,
            Err(DomainError::HasAssociatedBooks { .. })
        ));

        // user1's book_author row must be intact
        let user1_book = book_repository
            .find_by_id(&user1_id, book1.id())
            .await?
            .expect("user1's book must still exist");
        assert!(
            user1_book.author_ids().contains(&author_id),
            "user1's book_author row must be intact"
        );

        Ok(())
    }

    fn make_book(book_id_str: &str, author_ids: &[AuthorId]) -> Result<Book, DomainError> {
        let book_id = BookId::try_from(book_id_str)?;
        let title = BookTitle::new("title1".to_owned())?;
        let isbn = Isbn::new("1111111111116".to_owned())?;
        let read = ReadFlag::new(false);
        let owned = OwnedFlag::new(false);
        let priority = Priority::new(50)?;
        let format = BookFormat::EBook;
        let store = BookStore::Kindle;
        let created_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        let updated_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        Book::new(
            book_id,
            title,
            author_ids.to_vec(),
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            created_at,
            updated_at,
        )
    }

    async fn prepare_user(repository: &PgUserRepository, id: &str) -> Result<UserId, DomainError> {
        let user_id = UserId::new(String::from(id))?;
        let user = User::new(user_id.clone());
        repository.create(&user).await?;

        Ok(user_id)
    }

    // ---- event recording tests ----

    #[sqlx::test]
    async fn create_records_history(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new_with_yomi(
            author_id.clone(),
            AuthorName::new("author1".to_owned())?,
            "おーさー1".to_owned(),
            OffsetDateTime::UNIX_EPOCH,
        )?;

        create_author(&pool, &author_repository, &user_id, &author).await?;

        let (es_op,): (String,) =
            sqlx::query_as("SELECT operation FROM event_set WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(es_op, "create_author");

        let (ae_op, ae_name, ae_yomi): (String, String, String) =
            sqlx::query_as("SELECT operation, name, yomi FROM author_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(ae_op, "create");
        assert_eq!(ae_name, "author1");
        assert_eq!(ae_yomi, "おーさー1");

        Ok(())
    }

    #[sqlx::test]
    async fn update_records_post_update_state(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new_with_yomi(
            author_id.clone(),
            AuthorName::new("original".to_owned())?,
            "おりじなる".to_owned(),
            OffsetDateTime::UNIX_EPOCH,
        )?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        let updated = Author::new_with_yomi(
            author_id.clone(),
            AuthorName::new("updated".to_owned())?,
            "あっぷでーと2".to_owned(),
            OffsetDateTime::UNIX_EPOCH,
        )?;
        update_author(&pool, &author_repository, &user_id, &updated).await?;

        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT operation, name, yomi FROM author_event WHERE user_id = $1
             ORDER BY changed_at ASC",
        )
        .bind(user_id.as_str())
        .fetch_all(&pool)
        .await?;

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "create");
        assert_eq!(rows[0].1, "original");
        assert_eq!(rows[0].2, "おりじなる");
        // Post-state: update event records the new name
        assert_eq!(rows[1].0, "update");
        assert_eq!(rows[1].1, "updated");
        assert_eq!(rows[1].2, "あっぷでーと2");

        let es_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM event_set WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(es_count.0, 2);

        Ok(())
    }

    #[sqlx::test]
    async fn restore_persists_yomi_and_records_it(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("original".to_owned())?)?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        let (source_event_id,): (i64,) = sqlx::query_as(
            "SELECT event_id FROM author_event WHERE user_id = $1 AND operation = 'create'",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        let restored = Author::new_with_yomi(
            author_id.clone(),
            AuthorName::new("restored".to_owned())?,
            "れすとあ".to_owned(),
            OffsetDateTime::UNIX_EPOCH,
        )?;

        restore_author(
            &pool,
            &author_repository,
            &user_id,
            source_event_id,
            restored,
        )
        .await?;

        let live = author_repository
            .find_by_id(&user_id, &author_id)
            .await?
            .expect("restored author should exist");
        assert_eq!(live.name().as_str(), "restored");
        assert_eq!(live.yomi(), "れすとあ");

        let (event_name, event_yomi): (String, String) = sqlx::query_as(
            "SELECT name, yomi FROM author_event
             WHERE user_id = $1 AND operation = 'restore'",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(event_name, "restored");
        assert_eq!(event_yomi, "れすとあ");

        Ok(())
    }

    #[sqlx::test]
    async fn delete_records_event_with_id_only(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = new_author(author_id.clone(), AuthorName::new("author1".to_owned())?)?;
        create_author(&pool, &author_repository, &user_id, &author).await?;

        delete_author(&pool, &author_repository, &user_id, &author_id).await?;

        let rows: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT operation, name FROM author_event WHERE user_id = $1
             ORDER BY changed_at ASC",
        )
        .bind(user_id.as_str())
        .fetch_all(&pool)
        .await?;

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "create");
        // Delete event: only author_id stored, name is NULL
        assert_eq!(rows[1].0, "delete");
        assert_eq!(rows[1].1, None);

        Ok(())
    }

    #[sqlx::test]
    async fn find_or_create_by_name_inserts_new_author_and_records_event(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(&user_id, EventSetOperation::ImportBooks).await?;
        let name = AuthorName::new("New Author".to_owned())?;
        let created_at = OffsetDateTime::from_unix_timestamp(1_700_000_000)?;
        let author_id = author_repository
            .find_or_create_by_name(&mut tx, &name, created_at)
            .await?;
        tm.commit(tx).await?;

        // The author row exists with the returned id and name
        let found = author_repository.find_by_id(&user_id, &author_id).await?;
        let found = found.expect("new author exists");
        assert_eq!(found.name().as_str(), "New Author");
        assert_eq!(found.created_at(), &created_at);
        assert_eq!(found.updated_at(), &created_at);

        // Exactly one author_event was recorded for the newly inserted author
        let (author_event_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM author_event WHERE user_id = $1 AND author_id = $2",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .fetch_one(&pool)
        .await?;
        assert_eq!(author_event_count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn find_or_create_by_name_reuses_existing_author_without_event(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        // Pre-create the author through the ordinary create path
        let existing_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let existing = new_author(existing_id.clone(), AuthorName::new("Existing".to_owned())?)?;
        create_author(&pool, &author_repository, &user_id, &existing).await?;

        let (events_before,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;

        // find_or_create_by_name on the existing name returns the same id and
        // records no additional author_event
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(&user_id, EventSetOperation::ImportBooks).await?;
        let name = AuthorName::new("Existing".to_owned())?;
        let attempted_created_at = OffsetDateTime::from_unix_timestamp(1_700_000_000)?;
        let resolved = author_repository
            .find_or_create_by_name(&mut tx, &name, attempted_created_at)
            .await?;
        tm.commit(tx).await?;

        assert_eq!(resolved, existing_id);

        let (events_after,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(events_after, events_before);

        let found = author_repository
            .find_by_id(&user_id, &existing_id)
            .await?
            .expect("existing author remains");
        assert_eq!(found.created_at(), &OffsetDateTime::UNIX_EPOCH);
        assert_eq!(found.updated_at(), &OffsetDateTime::UNIX_EPOCH);

        Ok(())
    }
}
