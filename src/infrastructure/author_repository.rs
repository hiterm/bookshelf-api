use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        author::{Author, AuthorId, AuthorName},
        user::UserId,
    },
    error::DomainError,
    repository::author_repository::{
        AuthorRepository, DeleteAuthorExtra, FindOrCreateAuthorsResult,
    },
};
use crate::infrastructure::{
    history_recording::{
        append_author_deletion, append_author_revision, latest_author_revision_number,
    },
    transaction::PgTransaction,
};

#[derive(sqlx::FromRow)]
struct AuthorRow {
    id: Uuid,
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

#[derive(sqlx::FromRow)]
struct BulkAuthorRow {
    id: Uuid,
    name: String,
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

    async fn create(
        &self,
        tx: &mut Self::Transaction,
        author: &Author,
    ) -> Result<i32, DomainError> {
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

        let revision_number = append_author_revision(tx, author, None).await?;
        Ok(revision_number)
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
                "WITH inserted_revision AS (
                   INSERT INTO author_revision (
                     author_id, revision_number, user_id, name, yomi,
                     author_created_at, author_updated_at
                   ) VALUES ($1, 1, $2, $3, $4, $5, $6)
                   RETURNING author_id, revision_number
                 )
                 INSERT INTO author_operation_change (
                   operation_id, user_id, author_id, before_revision_number,
                   after_revision_number
                 )
                 SELECT $7, $2, author_id, NULL, revision_number
                 FROM inserted_revision",
            )
            .bind(author_id.to_uuid())
            .bind(user_id.as_str())
            .bind(name)
            .bind(&snap.yomi)
            .bind(snap.created_at)
            .bind(snap.updated_at)
            .bind(tx.operation_id().to_uuid())
            .execute(tx.as_mut())
            .await?;
        }

        Ok(author_id)
    }

    async fn find_or_create_by_names(
        &self,
        tx: &mut Self::Transaction,
        names: &[AuthorName],
        created_at: OffsetDateTime,
    ) -> Result<FindOrCreateAuthorsResult, DomainError> {
        if names.is_empty() {
            return Ok(FindOrCreateAuthorsResult {
                authors_by_name: HashMap::new(),
                created_author_ids: HashSet::new(),
            });
        }

        let user_id = tx.user_id().clone();
        let ids: Vec<Uuid> = names.iter().map(|_| Uuid::new_v4()).collect();
        let names: Vec<&str> = names.iter().map(AuthorName::as_str).collect();
        let created: Vec<BulkAuthorRow> = sqlx::query_as(
            "INSERT INTO author (id, user_id, name, created_at, updated_at)
             SELECT id, $1, name, $2, $2
             FROM UNNEST($3::uuid[], $4::text[]) AS input(id, name)
             ON CONFLICT (user_id, name) DO NOTHING
             RETURNING id, name, yomi, created_at, updated_at",
        )
        .bind(user_id.as_str())
        .bind(created_at)
        .bind(&ids)
        .bind(&names)
        .fetch_all(tx.as_mut())
        .await?;

        if !created.is_empty() {
            let author_ids: Vec<Uuid> = created.iter().map(|author| author.id).collect();
            let names: Vec<&str> = created.iter().map(|author| author.name.as_str()).collect();
            let yomis: Vec<&str> = created.iter().map(|author| author.yomi.as_str()).collect();
            let created_ats: Vec<OffsetDateTime> =
                created.iter().map(|author| author.created_at).collect();
            let updated_ats: Vec<OffsetDateTime> =
                created.iter().map(|author| author.updated_at).collect();
            sqlx::query(
                "WITH inserted_revisions AS (
                   INSERT INTO author_revision (
                     author_id, revision_number, user_id, name, yomi,
                     author_created_at, author_updated_at
                   )
                   SELECT author_id, 1, $1, name, yomi, created_at, updated_at
                   FROM UNNEST(
                     $2::uuid[], $3::text[], $4::text[], $5::timestamptz[],
                     $6::timestamptz[]
                   ) AS input(author_id, name, yomi, created_at, updated_at)
                   RETURNING author_id, revision_number
                 )
                 INSERT INTO author_operation_change (
                   operation_id, user_id, author_id, before_revision_number,
                   after_revision_number
                 )
                 SELECT $7, $1, author_id, NULL, revision_number
                 FROM inserted_revisions",
            )
            .bind(user_id.as_str())
            .bind(&author_ids)
            .bind(&names)
            .bind(&yomis)
            .bind(&created_ats)
            .bind(&updated_ats)
            .bind(tx.operation_id().to_uuid())
            .execute(tx.as_mut())
            .await?;
        }

        let resolved: Vec<(Uuid, String)> = sqlx::query_as(
            "SELECT id, name FROM author WHERE user_id = $1 AND name = ANY($2::text[])",
        )
        .bind(user_id.as_str())
        .bind(&names)
        .fetch_all(tx.as_mut())
        .await?;

        if resolved.len() != names.len() {
            return Err(DomainError::Unexpected(format!(
                "resolved {} authors for {} unique names",
                resolved.len(),
                names.len()
            )));
        }

        let created_author_ids = created
            .into_iter()
            .map(|author| AuthorId::new(author.id))
            .collect();
        let authors_by_name = resolved
            .into_iter()
            .map(|(id, name)| (name, AuthorId::new(id)))
            .collect();

        Ok(FindOrCreateAuthorsResult {
            authors_by_name,
            created_author_ids,
        })
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

    async fn update(
        &self,
        tx: &mut Self::Transaction,
        author: &Author,
    ) -> Result<i32, DomainError> {
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

        let before_revision_number =
            latest_author_revision_number(tx, author.id().to_uuid()).await?;

        let revision_number =
            append_author_revision(tx, author, Some(before_revision_number)).await?;
        Ok(revision_number)
    }

    async fn record_unchanged_revision(
        &self,
        tx: &mut Self::Transaction,
        author: &Author,
    ) -> Result<(), DomainError> {
        let before_revision_number =
            latest_author_revision_number(tx, author.id().to_uuid()).await?;
        append_author_revision(tx, author, Some(before_revision_number)).await?;
        Ok(())
    }

    async fn delete(
        &self,
        tx: &mut Self::Transaction,
        author_id: &AuthorId,
        extra: Option<DeleteAuthorExtra>,
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

        let before_revision_number = latest_author_revision_number(tx, author_id.to_uuid()).await?;

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

        let _ = extra;

        append_author_deletion(tx, author_id.to_uuid(), before_revision_number).await?;

        Ok(())
    }

    async fn restore_revision(
        &self,
        tx: &mut Self::Transaction,
        author_id: &AuthorId,
        revision_number: i32,
    ) -> Result<Author, DomainError> {
        let user_id = tx.user_id().clone();
        let source: Option<(String, String, OffsetDateTime)> = sqlx::query_as(
            "SELECT name, yomi, author_created_at
             FROM author_revision
             WHERE user_id = $1 AND author_id = $2 AND revision_number = $3
             FOR UPDATE",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .bind(revision_number)
        .fetch_optional(tx.as_mut())
        .await?;
        let (name, yomi, created_at) = source.ok_or_else(|| DomainError::NotFound {
            entity_type: "author_revision",
            entity_id: format!("{author_id}:{revision_number}"),
            user_id: user_id.as_str().to_owned(),
        })?;
        let before_revision_number: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(revision.revision_number)
             FROM author_revision revision
             WHERE revision.user_id = $1 AND revision.author_id = $2
               AND EXISTS (
                 SELECT 1 FROM author current
                 WHERE current.user_id = $1 AND current.id = $2
               )",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .fetch_one(tx.as_mut())
        .await?;
        let author = Author::new_with_timestamps(
            author_id.clone(),
            AuthorName::new(name)?,
            yomi,
            created_at,
            OffsetDateTime::now_utc(),
        )?;
        let conflicting_author_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
               SELECT 1 FROM author
               WHERE user_id = $1 AND name = $2 AND id <> $3
             )",
        )
        .bind(user_id.as_str())
        .bind(author.name().as_str())
        .bind(author.id().to_uuid())
        .fetch_one(tx.as_mut())
        .await?;
        if conflicting_author_exists {
            return Err(DomainError::Validation(format!(
                "author name '{}' is already in use",
                author.name().as_str()
            )));
        }
        let upsert_result = sqlx::query(
            "INSERT INTO author (id, user_id, name, yomi, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id, user_id) DO UPDATE SET
               name = EXCLUDED.name, yomi = EXCLUDED.yomi,
               created_at = EXCLUDED.created_at, updated_at = EXCLUDED.updated_at",
        )
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .bind(author.name().as_str())
        .bind(author.yomi())
        .bind(author.created_at())
        .bind(author.updated_at())
        .execute(tx.as_mut())
        .await;
        if let Err(sqlx::Error::Database(error)) = &upsert_result
            && error.constraint() == Some("author_user_id_name_unique")
        {
            return Err(DomainError::Validation(format!(
                "author name '{}' is already in use",
                author.name().as_str()
            )));
        }
        upsert_result?;
        append_author_revision(tx, &author, before_revision_number).await?;
        Ok(author)
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
                operation::NewOperation,
                operation::OperationType,
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
        let mut tx = tm.begin(user_id, OperationType::CreateBook).await?;
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
    ) -> Result<i64, DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, OperationType::CreateAuthor).await?;
        let event_id = author_repository.create(&mut tx, author).await?;
        tm.commit(tx).await?;
        Ok(i64::from(event_id))
    }

    async fn update_author(
        pool: &PgPool,
        author_repository: &PgAuthorRepository,
        user_id: &UserId,
        author: &Author,
    ) -> Result<i64, DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, OperationType::UpdateAuthor).await?;
        let event_id = author_repository.update(&mut tx, author).await?;
        tm.commit(tx).await?;
        Ok(i64::from(event_id))
    }

    async fn delete_author(
        pool: &PgPool,
        author_repository: &PgAuthorRepository,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, OperationType::DeleteAuthor).await?;
        author_repository.delete(&mut tx, author_id, None).await?;
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
        let mut tx = tm.begin(&user_id, OperationType::UpdateAuthor).await?;
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
        let mut tx = tm.begin(&user_id, OperationType::UpdateAuthor).await?;
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

    #[sqlx::test]
    async fn find_or_create_by_names_reports_created_ids_and_rolls_back(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let user_id = prepare_user(&user_repository, "bulk-status-user").await?;
        let existing_id = AuthorId::new(Uuid::new_v4());
        let existing = new_author(
            existing_id.clone(),
            AuthorName::new("Existing".to_string())?,
        )?;
        create_author(&pool, &author_repository, &user_id, &existing).await?;

        let manager = PgTransactionManager::new(pool.clone());
        let mut tx = manager.begin(&user_id, OperationType::ImportBooks).await?;
        let names = vec![
            AuthorName::new("Existing".to_string())?,
            AuthorName::new("New".to_string())?,
        ];
        let result = author_repository
            .find_or_create_by_names(&mut tx, &names, OffsetDateTime::now_utc())
            .await?;

        assert_eq!(result.authors_by_name["Existing"], existing_id);
        assert!(!result.created_author_ids.contains(&existing_id));
        let new_id = result.authors_by_name["New"].clone();
        assert!(result.created_author_ids.contains(&new_id));
        manager.rollback(tx).await?;

        assert!(
            author_repository
                .find_by_id(&user_id, &new_id)
                .await?
                .is_none()
        );
        assert!(
            author_repository
                .find_by_id(&user_id, &existing_id)
                .await?
                .is_some()
        );
        Ok(())
    }

    #[sqlx::test]
    async fn restore_revision_appends_fresh_owned_revision(pool: PgPool) -> anyhow::Result<()> {
        let users = PgUserRepository::new(pool.clone());
        let repository = PgAuthorRepository::new(pool.clone());
        let user_id = prepare_user(&users, "restore-user").await?;
        let author_id = AuthorId::new(Uuid::new_v4());
        let original = new_author(author_id.clone(), AuthorName::new("Original".to_string())?)?;
        create_author(&pool, &repository, &user_id, &original).await?;
        let mut changed = original.clone();
        changed.update(
            crate::domain::entity::author::AuthorUpdate {
                name: AuthorName::new("Changed".to_string())?,
                yomi: Some(String::new()),
            },
            OffsetDateTime::now_utc(),
        );
        update_author(&pool, &repository, &user_id, &changed).await?;

        let manager = PgTransactionManager::new(pool.clone());
        let mut tx = manager
            .begin_operation(&user_id, &NewOperation::restore_author(1))
            .await?;
        let restored = repository.restore_revision(&mut tx, &author_id, 1).await?;
        let operation_id = tx.operation_id();
        manager.commit(tx).await?;

        assert_eq!(restored.name().as_str(), "Original");
        let change: (Option<i32>, Option<i32>) = sqlx::query_as(
            "SELECT before_revision_number, after_revision_number
             FROM author_operation_change WHERE operation_id = $1 AND user_id = $2",
        )
        .bind(operation_id.to_uuid())
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(change, (Some(2), Some(3)));
        Ok(())
    }

    #[sqlx::test]
    async fn restore_revision_rejects_name_owned_by_another_author(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let users = PgUserRepository::new(pool.clone());
        let repository = PgAuthorRepository::new(pool.clone());
        let user_id = prepare_user(&users, "restore-conflict-user").await?;
        let author_id = AuthorId::new(Uuid::new_v4());
        let original = new_author(author_id.clone(), AuthorName::new("Original".to_string())?)?;
        create_author(&pool, &repository, &user_id, &original).await?;
        let mut changed = original.clone();
        changed.update(
            crate::domain::entity::author::AuthorUpdate {
                name: AuthorName::new("Changed".to_string())?,
                yomi: Some(String::new()),
            },
            OffsetDateTime::now_utc(),
        );
        update_author(&pool, &repository, &user_id, &changed).await?;
        let conflicting = new_author(
            AuthorId::new(Uuid::new_v4()),
            AuthorName::new("Original".to_string())?,
        )?;
        create_author(&pool, &repository, &user_id, &conflicting).await?;

        let manager = PgTransactionManager::new(pool);
        let mut tx = manager
            .begin_operation(&user_id, &NewOperation::restore_author(1))
            .await?;
        let result = repository.restore_revision(&mut tx, &author_id, 1).await;

        assert!(matches!(result, Err(DomainError::Validation(_))));
        manager.rollback(tx).await?;
        Ok(())
    }
}
