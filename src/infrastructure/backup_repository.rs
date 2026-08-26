use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    domain::{
        entity::user::UserId, error::DomainError, repository::backup_repository::BackupRepository,
    },
    use_case::dto::backup::{
        BackupAuthorEventV1, BackupAuthorV1, BackupBookEventV1, BackupBookV1, BackupEventSetV1,
        BackupHistoryV1, CurrentBackupDataV1,
    },
};

use super::transaction::acquire_user_lock;

#[derive(Clone, Debug)]
pub struct PgBackupRepository {
    pool: PgPool,
}

impl PgBackupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AuthorRow {
    id: Uuid,
    name: String,
    yomi: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct BookRow {
    id: Uuid,
    title: String,
    isbn: String,
    read: bool,
    owned: bool,
    priority: i32,
    format: String,
    store: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    author_ids: Vec<Uuid>,
}

#[derive(sqlx::FromRow)]
struct EventSetRow {
    id: Uuid,
    operation: String,
    created_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct BookEventRow {
    event_id: i64,
    event_set_id: Uuid,
    operation: String,
    book_id: Uuid,
    title: Option<String>,
    isbn: Option<String>,
    read: Option<bool>,
    owned: Option<bool>,
    priority: Option<i32>,
    format: Option<String>,
    store: Option<String>,
    book_created_at: Option<OffsetDateTime>,
    book_updated_at: Option<OffsetDateTime>,
    changed_at: OffsetDateTime,
    extra: Option<Value>,
    author_ids: Vec<Uuid>,
}

#[derive(sqlx::FromRow)]
struct AuthorEventRow {
    event_id: i64,
    event_set_id: Uuid,
    operation: String,
    author_id: Uuid,
    name: Option<String>,
    yomi: Option<String>,
    author_created_at: Option<OffsetDateTime>,
    author_updated_at: Option<OffsetDateTime>,
    changed_at: OffsetDateTime,
    extra: Option<Value>,
}

fn timestamp(value: OffsetDateTime) -> Result<String, DomainError> {
    value
        .format(&Rfc3339)
        .map_err(|error| DomainError::Unexpected(error.to_string()))
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime, DomainError> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|error| DomainError::Validation(error.to_string()))
}

fn parse_uuid(value: &str) -> Result<Uuid, DomainError> {
    Uuid::parse_str(value).map_err(|error| DomainError::Validation(error.to_string()))
}

async fn current_data(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
) -> Result<CurrentBackupDataV1, DomainError> {
    let authors: Vec<AuthorRow> = sqlx::query_as(
        "SELECT id, name, yomi, created_at, updated_at FROM author WHERE user_id = $1 ORDER BY id",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    let books: Vec<BookRow> = sqlx::query_as(
        "SELECT b.id, b.title, b.isbn, b.read, b.owned, b.priority, b.format, b.store,
                b.created_at, b.updated_at,
                COALESCE(array_agg(ba.author_id ORDER BY ba.author_id)
                  FILTER (WHERE ba.author_id IS NOT NULL), ARRAY[]::uuid[]) author_ids
         FROM book b LEFT JOIN book_author ba ON ba.user_id = b.user_id AND ba.book_id = b.id
         WHERE b.user_id = $1 GROUP BY b.id, b.user_id ORDER BY b.id",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    Ok(CurrentBackupDataV1 {
        authors: authors
            .into_iter()
            .map(|row| {
                Ok(BackupAuthorV1 {
                    id: row.id.to_string(),
                    name: row.name,
                    yomi: row.yomi,
                    created_at: timestamp(row.created_at)?,
                    updated_at: timestamp(row.updated_at)?,
                })
            })
            .collect::<Result<_, DomainError>>()?,
        books: books
            .into_iter()
            .map(|row| {
                Ok(BackupBookV1 {
                    id: row.id.to_string(),
                    title: row.title,
                    isbn: row.isbn,
                    read: row.read,
                    owned: row.owned,
                    priority: row.priority,
                    format: row.format,
                    store: row.store,
                    created_at: timestamp(row.created_at)?,
                    updated_at: timestamp(row.updated_at)?,
                    author_ids: row
                        .author_ids
                        .into_iter()
                        .map(|id| id.to_string())
                        .collect(),
                })
            })
            .collect::<Result<_, DomainError>>()?,
    })
}

async fn history(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
) -> Result<BackupHistoryV1, DomainError> {
    let event_sets: Vec<EventSetRow> = sqlx::query_as(
        "SELECT id, operation, created_at FROM event_set WHERE user_id = $1 ORDER BY created_at, id",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    let book_events: Vec<BookEventRow> = sqlx::query_as(
        "SELECT be.event_id, be.event_set_id, be.operation, be.book_id, be.title, be.isbn,
                be.read, be.owned, be.priority, be.format, be.store, be.book_created_at,
                be.book_updated_at, be.changed_at, be.extra,
                COALESCE(array_agg(bea.author_id ORDER BY bea.author_id)
                  FILTER (WHERE bea.author_id IS NOT NULL), ARRAY[]::uuid[]) author_ids
         FROM book_event be LEFT JOIN book_event_author bea ON bea.event_id = be.event_id
         WHERE be.user_id = $1 GROUP BY be.event_id ORDER BY be.event_id",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;
    let author_events: Vec<AuthorEventRow> = sqlx::query_as(
        "SELECT event_id, event_set_id, operation, author_id, name, yomi, author_created_at,
                author_updated_at, changed_at, extra
         FROM author_event WHERE user_id = $1 ORDER BY event_id",
    )
    .bind(user_id.as_str())
    .fetch_all(&mut **tx)
    .await?;

    Ok(BackupHistoryV1 {
        event_sets: event_sets
            .into_iter()
            .map(|row| {
                Ok(BackupEventSetV1 {
                    id: row.id.to_string(),
                    operation: row.operation,
                    created_at: timestamp(row.created_at)?,
                })
            })
            .collect::<Result<_, DomainError>>()?,
        book_events: book_events
            .into_iter()
            .map(|row| {
                Ok(BackupBookEventV1 {
                    event_id: row.event_id,
                    event_set_id: row.event_set_id.to_string(),
                    operation: row.operation,
                    book_id: row.book_id.to_string(),
                    title: row.title,
                    isbn: row.isbn,
                    read: row.read,
                    owned: row.owned,
                    priority: row.priority,
                    format: row.format,
                    store: row.store,
                    created_at: row.book_created_at.map(timestamp).transpose()?,
                    updated_at: row.book_updated_at.map(timestamp).transpose()?,
                    author_ids: row
                        .author_ids
                        .into_iter()
                        .map(|id| id.to_string())
                        .collect(),
                    changed_at: timestamp(row.changed_at)?,
                    extra: row.extra,
                })
            })
            .collect::<Result<_, DomainError>>()?,
        author_events: author_events
            .into_iter()
            .map(|row| {
                Ok(BackupAuthorEventV1 {
                    event_id: row.event_id,
                    event_set_id: row.event_set_id.to_string(),
                    operation: row.operation,
                    author_id: row.author_id.to_string(),
                    name: row.name,
                    yomi: row.yomi,
                    created_at: row.author_created_at.map(timestamp).transpose()?,
                    updated_at: row.author_updated_at.map(timestamp).transpose()?,
                    changed_at: timestamp(row.changed_at)?,
                    extra: row.extra,
                })
            })
            .collect::<Result<_, DomainError>>()?,
    })
}

async fn insert_current(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    data: &CurrentBackupDataV1,
) -> Result<(), DomainError> {
    for author in &data.authors {
        sqlx::query("INSERT INTO author (id,user_id,name,yomi,created_at,updated_at) VALUES ($1,$2,$3,$4,$5,$6)")
            .bind(parse_uuid(&author.id)?).bind(user_id.as_str()).bind(&author.name).bind(&author.yomi)
            .bind(parse_timestamp(&author.created_at)?).bind(parse_timestamp(&author.updated_at)?)
            .execute(&mut **tx).await?;
    }
    for book in &data.books {
        let book_id = parse_uuid(&book.id)?;
        sqlx::query("INSERT INTO book (id,user_id,title,isbn,read,owned,priority,format,store,created_at,updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
            .bind(book_id).bind(user_id.as_str()).bind(&book.title).bind(&book.isbn).bind(book.read)
            .bind(book.owned).bind(book.priority).bind(&book.format).bind(&book.store)
            .bind(parse_timestamp(&book.created_at)?).bind(parse_timestamp(&book.updated_at)?)
            .execute(&mut **tx).await?;
        for author_id in &book.author_ids {
            sqlx::query("INSERT INTO book_author (user_id,book_id,author_id) VALUES ($1,$2,$3)")
                .bind(user_id.as_str())
                .bind(book_id)
                .bind(parse_uuid(author_id)?)
                .execute(&mut **tx)
                .await?;
        }
    }
    Ok(())
}

async fn delete_current(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
) -> Result<(), DomainError> {
    for statement in [
        "DELETE FROM book_author WHERE user_id=$1",
        "DELETE FROM book WHERE user_id=$1",
        "DELETE FROM author WHERE user_id=$1",
    ] {
        sqlx::query(statement)
            .bind(user_id.as_str())
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn snapshot(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    phase: &str,
) -> Result<(), DomainError> {
    let event_set_id = Uuid::new_v4();
    sqlx::query("INSERT INTO event_set (id,user_id,operation) VALUES ($1,$2,'snapshot_all')")
        .bind(event_set_id)
        .bind(user_id.as_str())
        .execute(&mut **tx)
        .await?;
    let extra = json!({"version":1,"reason":"current_backup_restore","phase":phase});
    let inserted: Vec<(i64, Uuid)> = sqlx::query_as(
        "INSERT INTO book_event (event_set_id,operation,book_id,user_id,title,isbn,read,owned,priority,format,store,book_created_at,book_updated_at,extra)
         SELECT $1,'snapshot',id,user_id,title,isbn,read,owned,priority,format,store,created_at,updated_at,$3 FROM book WHERE user_id=$2 RETURNING event_id,book_id")
        .bind(event_set_id).bind(user_id.as_str()).bind(&extra).fetch_all(&mut **tx).await?;
    for (event_id, book_id) in inserted {
        sqlx::query("INSERT INTO book_event_author (event_id,author_id) SELECT $1,author_id FROM book_author WHERE user_id=$2 AND book_id=$3")
            .bind(event_id).bind(user_id.as_str()).bind(book_id).execute(&mut **tx).await?;
    }
    sqlx::query("INSERT INTO author_event (event_set_id,operation,author_id,user_id,name,yomi,author_created_at,author_updated_at,extra) SELECT $1,'snapshot',id,user_id,name,yomi,created_at,updated_at,$3 FROM author WHERE user_id=$2")
        .bind(event_set_id).bind(user_id.as_str()).bind(&extra).execute(&mut **tx).await?;
    Ok(())
}

fn remapped_extra(
    extra: Option<&Value>,
    mapping: &HashMap<i64, i64>,
) -> Result<Option<Value>, DomainError> {
    let Some(mut extra) = extra.cloned() else {
        return Ok(None);
    };
    if let Some(source) = extra.get("source_event_id").and_then(Value::as_i64) {
        let mapped = mapping
            .get(&source)
            .ok_or_else(|| DomainError::Validation(format!("unknown source event {source}")))?;
        extra["source_event_id"] = Value::from(*mapped);
        return Ok(Some(extra));
    }
    Ok(None)
}

#[async_trait]
impl BackupRepository for PgBackupRepository {
    async fn export_current(&self, user_id: &UserId) -> Result<CurrentBackupDataV1, DomainError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await?;
        let data = current_data(&mut tx, user_id).await?;
        tx.commit().await?;
        Ok(data)
    }

    async fn export_full(
        &self,
        user_id: &UserId,
    ) -> Result<(CurrentBackupDataV1, BackupHistoryV1), DomainError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await?;
        let data = current_data(&mut tx, user_id).await?;
        let history = history(&mut tx, user_id).await?;
        tx.commit().await?;
        Ok((data, history))
    }

    async fn restore_current(
        &self,
        user_id: &UserId,
        data: &CurrentBackupDataV1,
    ) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await?;
        acquire_user_lock(&mut tx, user_id).await?;
        snapshot(&mut tx, user_id, "before").await?;
        delete_current(&mut tx, user_id).await?;
        insert_current(&mut tx, user_id, data).await?;
        snapshot(&mut tx, user_id, "after").await?;
        tx.commit().await?;
        Ok(())
    }

    async fn restore_full(
        &self,
        user_id: &UserId,
        data: &CurrentBackupDataV1,
        history_data: &BackupHistoryV1,
    ) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await?;
        acquire_user_lock(&mut tx, user_id).await?;
        sqlx::query("DELETE FROM book_event WHERE user_id=$1")
            .bind(user_id.as_str())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM author_event WHERE user_id=$1")
            .bind(user_id.as_str())
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM event_set WHERE user_id=$1")
            .bind(user_id.as_str())
            .execute(&mut *tx)
            .await?;
        delete_current(&mut tx, user_id).await?;
        insert_current(&mut tx, user_id, data).await?;
        for set in &history_data.event_sets {
            sqlx::query(
                "INSERT INTO event_set (id,user_id,operation,created_at) VALUES ($1,$2,$3,$4)",
            )
            .bind(parse_uuid(&set.id)?)
            .bind(user_id.as_str())
            .bind(&set.operation)
            .bind(parse_timestamp(&set.created_at)?)
            .execute(&mut *tx)
            .await?;
        }
        let mut author_mapping = HashMap::new();
        for event in &history_data.author_events {
            let new_id: i64 = sqlx::query_scalar("INSERT INTO author_event (event_set_id,operation,author_id,user_id,name,yomi,author_created_at,author_updated_at,changed_at,extra) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING event_id")
                .bind(parse_uuid(&event.event_set_id)?).bind(&event.operation).bind(parse_uuid(&event.author_id)?).bind(user_id.as_str())
                .bind(&event.name).bind(&event.yomi).bind(event.created_at.as_deref().map(parse_timestamp).transpose()?)
                .bind(event.updated_at.as_deref().map(parse_timestamp).transpose()?).bind(parse_timestamp(&event.changed_at)?)
                .bind(&event.extra).fetch_one(&mut *tx).await?;
            author_mapping.insert(event.event_id, new_id);
        }
        let mut book_mapping = HashMap::new();
        for event in &history_data.book_events {
            let new_id: i64 = sqlx::query_scalar("INSERT INTO book_event (event_set_id,operation,book_id,user_id,title,isbn,read,owned,priority,format,store,book_created_at,book_updated_at,changed_at,extra) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15) RETURNING event_id")
                .bind(parse_uuid(&event.event_set_id)?).bind(&event.operation).bind(parse_uuid(&event.book_id)?).bind(user_id.as_str())
                .bind(&event.title).bind(&event.isbn).bind(event.read).bind(event.owned).bind(event.priority).bind(&event.format).bind(&event.store)
                .bind(event.created_at.as_deref().map(parse_timestamp).transpose()?).bind(event.updated_at.as_deref().map(parse_timestamp).transpose()?)
                .bind(parse_timestamp(&event.changed_at)?).bind(&event.extra).fetch_one(&mut *tx).await?;
            book_mapping.insert(event.event_id, new_id);
            for author_id in &event.author_ids {
                sqlx::query("INSERT INTO book_event_author (event_id,author_id) VALUES ($1,$2)")
                    .bind(new_id)
                    .bind(parse_uuid(author_id)?)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        for event in &history_data.author_events {
            if let Some(extra) = remapped_extra(event.extra.as_ref(), &author_mapping)? {
                sqlx::query("UPDATE author_event SET extra=$1 WHERE event_id=$2")
                    .bind(extra)
                    .bind(author_mapping[&event.event_id])
                    .execute(&mut *tx)
                    .await?;
            }
        }
        for event in &history_data.book_events {
            if let Some(extra) = remapped_extra(event.extra.as_ref(), &book_mapping)? {
                sqlx::query("UPDATE book_event SET extra=$1 WHERE event_id=$2")
                    .bind(extra)
                    .bind(book_mapping[&event.event_id])
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remapped_extra_skips_missing_extra() {
        assert_eq!(remapped_extra(None, &HashMap::new()).unwrap(), None);
    }

    #[test]
    fn remapped_extra_skips_extra_without_event_reference() {
        let extra = json!({"version": 1, "reason": "current_backup_restore"});
        assert_eq!(remapped_extra(Some(&extra), &HashMap::new()).unwrap(), None);
    }

    #[test]
    fn remapped_extra_replaces_source_event_id() {
        let extra = json!({"version": 1, "source_event_id": 10});
        let mapping = HashMap::from([(10, 42)]);
        assert_eq!(
            remapped_extra(Some(&extra), &mapping).unwrap(),
            Some(json!({"version": 1, "source_event_id": 42}))
        );
    }

    #[test]
    fn remapped_extra_rejects_unknown_source_event() {
        let extra = json!({"version": 1, "source_event_id": 10});
        assert!(matches!(
            remapped_extra(Some(&extra), &HashMap::new()),
            Err(DomainError::Validation(_))
        ));
    }
}
