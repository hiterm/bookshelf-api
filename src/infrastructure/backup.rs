use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use time::{Date, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    domain::{
        entity::user::UserId, error::DomainError, repository::backup_repository::BackupRepository,
    },
    use_case::dto::backup::{
        BackupAuthor, BackupAuthorRevision, BackupAuthorSnapshot, BackupBook, BackupBookRevision,
        BackupBookSnapshot, BackupChanges, BackupData, BackupEntityChange, BackupHistory,
        BackupOperation, BackupScope,
    },
};

#[derive(Debug, Clone)]
pub struct PgBackupRepository {
    pool: PgPool,
}

impl PgBackupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct AuthorRow {
    id: Uuid,
    name: String,
    yomi: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(FromRow)]
struct BookRow {
    id: Uuid,
    title: String,
    author_ids: Vec<Uuid>,
    isbn: String,
    read: bool,
    owned: bool,
    priority: i32,
    format: String,
    store: String,
    purchase_date: Option<Date>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(FromRow)]
struct OperationRow {
    id: Uuid,
    operation_type: String,
    detail: Option<serde_json::Value>,
    undo_of_operation_id: Option<Uuid>,
    created_at: OffsetDateTime,
}

#[derive(FromRow)]
struct ChangeRow {
    operation_id: Uuid,
    entity_id: Uuid,
    before_revision_number: Option<i32>,
    after_revision_number: Option<i32>,
}

#[derive(FromRow)]
struct BookRevisionRow {
    book_id: Uuid,
    revision_number: i32,
    title: String,
    author_ids: Vec<Uuid>,
    isbn: String,
    read: bool,
    owned: bool,
    priority: i32,
    format: String,
    store: String,
    purchase_date: Option<Date>,
    book_created_at: OffsetDateTime,
    book_updated_at: OffsetDateTime,
    recorded_at: OffsetDateTime,
}

#[derive(FromRow)]
struct AuthorRevisionRow {
    author_id: Uuid,
    revision_number: i32,
    name: String,
    yomi: String,
    author_created_at: OffsetDateTime,
    author_updated_at: OffsetDateTime,
    recorded_at: OffsetDateTime,
}

fn timestamp(value: OffsetDateTime) -> Result<String, DomainError> {
    value
        .format(&Rfc3339)
        .map_err(|error| DomainError::Unexpected(error.to_string()))
}

fn format_name(value: &str) -> Result<String, DomainError> {
    match value {
        "eBook" => Ok("E_BOOK".to_owned()),
        "Printed" => Ok("PRINTED".to_owned()),
        "Unknown" => Ok("UNKNOWN".to_owned()),
        other => Err(DomainError::Unexpected(format!(
            "unsupported persisted book format: {other}"
        ))),
    }
}

fn store_name(value: &str) -> Result<String, DomainError> {
    match value {
        "Kindle" => Ok("KINDLE".to_owned()),
        "Unknown" => Ok("UNKNOWN".to_owned()),
        other => Err(DomainError::Unexpected(format!(
            "unsupported persisted book store: {other}"
        ))),
    }
}

fn book_snapshot(row: &BookRow) -> Result<BackupBookSnapshot, DomainError> {
    Ok(BackupBookSnapshot {
        title: row.title.clone(),
        author_ids: row.author_ids.iter().map(Uuid::to_string).collect(),
        isbn: row.isbn.clone(),
        read: row.read,
        owned: row.owned,
        priority: row.priority,
        format: format_name(&row.format)?,
        store: store_name(&row.store)?,
        purchase_date: row.purchase_date.map(|date| date.to_string()),
        created_at: timestamp(row.created_at)?,
        updated_at: timestamp(row.updated_at)?,
    })
}

async fn current_data(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
) -> Result<(Vec<BackupAuthor>, Vec<BackupBook>), DomainError> {
    let authors = sqlx::query_as::<_, AuthorRow>(
        "SELECT id, name, yomi, created_at, updated_at FROM author \
         WHERE user_id = $1 ORDER BY id ASC",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok(BackupAuthor {
            id: row.id.to_string(),
            name: row.name,
            yomi: row.yomi,
            created_at: timestamp(row.created_at)?,
            updated_at: timestamp(row.updated_at)?,
        })
    })
    .collect::<Result<Vec<_>, DomainError>>()?;

    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT b.id, b.title, ARRAY(SELECT ba.author_id FROM book_author ba \
           WHERE ba.user_id = b.user_id AND ba.book_id = b.id ORDER BY ba.author_id) author_ids, \
         b.isbn, b.read, b.owned, b.priority, b.format, b.store, b.purchase_date, \
         b.created_at, b.updated_at FROM book b WHERE b.user_id = $1 ORDER BY b.id ASC",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    let books = rows
        .iter()
        .map(|row| {
            Ok(BackupBook {
                id: row.id.to_string(),
                snapshot: book_snapshot(row)?,
            })
        })
        .collect::<Result<Vec<_>, DomainError>>()?;
    Ok((authors, books))
}

async fn history(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
) -> Result<BackupHistory, DomainError> {
    let operation_rows = sqlx::query_as::<_, OperationRow>(
        "SELECT id, type operation_type, detail, undo_of_operation_id, created_at \
         FROM operation WHERE user_id = $1 ORDER BY created_at ASC, id ASC",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    let book_changes = sqlx::query_as::<_, ChangeRow>(
        "SELECT operation_id, book_id entity_id, before_revision_number, \
         after_revision_number FROM book_operation_change WHERE user_id = $1 \
         ORDER BY operation_id ASC, book_id ASC",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    let author_changes = sqlx::query_as::<_, ChangeRow>(
        "SELECT operation_id, author_id entity_id, before_revision_number, \
         after_revision_number FROM author_operation_change WHERE user_id = $1 \
         ORDER BY operation_id ASC, author_id ASC",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;

    let mut changes = HashMap::<Uuid, BackupChanges>::new();
    for row in book_changes {
        changes
            .entry(row.operation_id)
            .or_default()
            .books
            .push(BackupEntityChange {
                book_id: Some(row.entity_id.to_string()),
                author_id: None,
                before_revision_number: row.before_revision_number,
                after_revision_number: row.after_revision_number,
            });
    }
    for row in author_changes {
        changes
            .entry(row.operation_id)
            .or_default()
            .authors
            .push(BackupEntityChange {
                book_id: None,
                author_id: Some(row.entity_id.to_string()),
                before_revision_number: row.before_revision_number,
                after_revision_number: row.after_revision_number,
            });
    }
    let operations = operation_rows
        .into_iter()
        .map(|row| {
            Ok(BackupOperation {
                id: row.id.to_string(),
                operation_type: row.operation_type,
                detail: row.detail,
                undo_of_operation_id: row.undo_of_operation_id.map(|id| id.to_string()),
                created_at: timestamp(row.created_at)?,
                changes: changes.remove(&row.id).unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>, DomainError>>()?;
    if !changes.is_empty() {
        return Err(DomainError::Unexpected(
            "backup contains changes without an operation".to_owned(),
        ));
    }

    let book_rows = sqlx::query_as::<_, BookRevisionRow>(
        "SELECT br.book_id, br.revision_number, br.title, \
         ARRAY(SELECT bra.author_id FROM book_revision_author bra WHERE \
           bra.user_id = br.user_id AND bra.book_id = br.book_id AND \
           bra.revision_number = br.revision_number ORDER BY bra.author_id) author_ids, \
         br.isbn, br.read, br.owned, br.priority, br.format, br.store, br.purchase_date, \
         br.book_created_at, br.book_updated_at, br.created_at recorded_at \
         FROM book_revision br WHERE br.user_id = $1 \
         ORDER BY br.book_id ASC, br.revision_number ASC",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    let book_revisions = book_rows
        .into_iter()
        .map(|row| {
            Ok(BackupBookRevision {
                book_id: row.book_id.to_string(),
                revision_number: row.revision_number,
                snapshot: BackupBookSnapshot {
                    title: row.title,
                    author_ids: row.author_ids.iter().map(Uuid::to_string).collect(),
                    isbn: row.isbn,
                    read: row.read,
                    owned: row.owned,
                    priority: row.priority,
                    format: format_name(&row.format)?,
                    store: store_name(&row.store)?,
                    purchase_date: row.purchase_date.map(|date| date.to_string()),
                    created_at: timestamp(row.book_created_at)?,
                    updated_at: timestamp(row.book_updated_at)?,
                },
                recorded_at: timestamp(row.recorded_at)?,
            })
        })
        .collect::<Result<Vec<_>, DomainError>>()?;

    let author_revisions = sqlx::query_as::<_, AuthorRevisionRow>(
        "SELECT author_id, revision_number, name, yomi, author_created_at, \
         author_updated_at, created_at recorded_at FROM author_revision \
         WHERE user_id = $1 ORDER BY author_id ASC, revision_number ASC",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok(BackupAuthorRevision {
            author_id: row.author_id.to_string(),
            revision_number: row.revision_number,
            snapshot: BackupAuthorSnapshot {
                name: row.name,
                yomi: row.yomi,
                created_at: timestamp(row.author_created_at)?,
                updated_at: timestamp(row.author_updated_at)?,
            },
            recorded_at: timestamp(row.recorded_at)?,
        })
    })
    .collect::<Result<Vec<_>, DomainError>>()?;

    Ok(BackupHistory {
        operations,
        book_revisions,
        author_revisions,
    })
}

#[async_trait]
impl BackupRepository for PgBackupRepository {
    async fn export(
        &self,
        user_id: &UserId,
        scope: BackupScope,
    ) -> Result<BackupData, DomainError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await?;
        let (authors, books) = current_data(&mut tx, user_id).await?;
        let history = match scope {
            BackupScope::Snapshot => None,
            BackupScope::Full => Some(history(&mut tx, user_id).await?),
        };
        tx.commit().await?;
        Ok(BackupData {
            authors,
            books,
            history,
        })
    }
}

#[cfg(all(test, feature = "test-with-database"))]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        domain::{entity::user::UserId, repository::backup_repository::BackupRepository},
        infrastructure::backup_repository::PgBackupRepository,
        use_case::dto::backup::BackupScope,
    };

    async fn insert_user(pool: &PgPool, id: &str) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO bookshelf_user (id) VALUES ($1)")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[sqlx::test]
    async fn snapshot_is_ordered_tenant_scoped_and_has_no_history(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        insert_user(&pool, "owner").await?;
        insert_user(&pool, "other").await?;
        let first_author = Uuid::parse_str("00000000-0000-0000-0000-000000000001")?;
        let second_author = Uuid::parse_str("00000000-0000-0000-0000-000000000002")?;
        let book = Uuid::parse_str("00000000-0000-0000-0000-000000000010")?;
        let other_book = Uuid::parse_str("00000000-0000-0000-0000-000000000020")?;
        for (id, name) in [(second_author, "second"), (first_author, "first")] {
            sqlx::query(
                "INSERT INTO author (id, user_id, name, yomi) VALUES ($1, 'owner', $2, '')",
            )
            .bind(id)
            .bind(name)
            .execute(&pool)
            .await?;
        }
        sqlx::query("INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store, purchase_date) VALUES ($1, 'owner', 'book', '', true, false, 42, 'eBook', 'Kindle', DATE '2026-09-01')")
            .bind(book)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book_author (user_id, book_id, author_id) VALUES ('owner', $1, $2), ('owner', $1, $3)")
            .bind(book)
            .bind(second_author)
            .bind(first_author)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store) VALUES ($1, 'other', 'secret', '', false, false, 0, 'Unknown', 'Unknown')")
            .bind(other_book)
            .execute(&pool)
            .await?;

        let data = PgBackupRepository::new(pool)
            .export(&UserId::new("owner".to_owned())?, BackupScope::Snapshot)
            .await?;

        assert_eq!(data.authors.len(), 2);
        assert_eq!(data.authors[0].id, first_author.to_string());
        assert_eq!(data.books.len(), 1);
        assert_eq!(
            data.books[0].snapshot.author_ids,
            vec![first_author.to_string(), second_author.to_string()]
        );
        assert_eq!(
            data.books[0].snapshot.purchase_date.as_deref(),
            Some("2026-09-01")
        );
        assert_eq!(data.books[0].snapshot.format, "E_BOOK");
        assert!(data.history.is_none());
        assert!(!serde_json::to_string(&data)?.contains("secret"));
        Ok(())
    }

    #[sqlx::test]
    async fn full_includes_baseline_revisions_changes_detail_and_undo(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        insert_user(&pool, "owner").await?;
        insert_user(&pool, "other").await?;
        let book = Uuid::parse_str("00000000-0000-0000-0000-000000000010")?;
        let author = Uuid::parse_str("00000000-0000-0000-0000-000000000001")?;
        let baseline = Uuid::parse_str("00000000-0000-0000-0000-000000000100")?;
        let undo = Uuid::parse_str("00000000-0000-0000-0000-000000000101")?;
        sqlx::query("INSERT INTO operation (id, user_id, type, detail, created_at) VALUES ($1, 'owner', 'baseline', NULL, '2026-01-01T00:00:00Z'), ($2, 'owner', 'undo', '{\"type\":\"import_books\",\"imported_count\":1}', '2026-01-02T00:00:00Z')")
            .bind(baseline)
            .bind(undo)
            .execute(&pool)
            .await?;
        sqlx::query("UPDATE operation SET undo_of_operation_id = $1 WHERE id = $2")
            .bind(baseline)
            .bind(undo)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book_revision (book_id, revision_number, user_id, title, isbn, read, owned, priority, format, store, book_created_at, book_updated_at, purchase_date) VALUES ($1, 1, 'owner', 'deleted', '', false, true, 1, 'Printed', 'Unknown', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', NULL)")
            .bind(book)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO author_revision (author_id, revision_number, user_id, name, yomi, author_created_at, author_updated_at) VALUES ($1, 1, 'owner', 'author', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')")
            .bind(author)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book_revision_author (user_id, book_id, revision_number, author_id) VALUES ('owner', $1, 1, $2)")
            .bind(book)
            .bind(author)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book_operation_change (operation_id, user_id, book_id, before_revision_number, after_revision_number) VALUES ($1, 'owner', $2, NULL, 1), ($3, 'owner', $2, 1, NULL)")
            .bind(baseline)
            .bind(book)
            .bind(undo)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO author_operation_change (operation_id, user_id, author_id, before_revision_number, after_revision_number) VALUES ($1, 'owner', $2, NULL, 1)")
            .bind(baseline)
            .bind(author)
            .execute(&pool)
            .await?;
        let other_operation = Uuid::new_v4();
        sqlx::query("INSERT INTO operation (id, user_id, type) VALUES ($1, 'other', 'baseline')")
            .bind(other_operation)
            .execute(&pool)
            .await?;

        let data = PgBackupRepository::new(pool)
            .export(&UserId::new("owner".to_owned())?, BackupScope::Full)
            .await?;
        let history = data.history.expect("full history");
        assert_eq!(history.operations.len(), 2);
        assert_eq!(history.operations[0].operation_type, "baseline");
        assert_eq!(
            history.operations[0].changes.books[0].after_revision_number,
            Some(1)
        );
        assert_eq!(
            history.operations[1].undo_of_operation_id.as_deref(),
            Some(baseline.to_string().as_str())
        );
        assert!(history.operations[1].detail.is_some());
        assert_eq!(
            history.book_revisions[0].snapshot.author_ids,
            vec![author.to_string()]
        );
        assert_eq!(history.author_revisions.len(), 1);
        assert!(!serde_json::to_string(&history)?.contains("other"));
        Ok(())
    }
}
