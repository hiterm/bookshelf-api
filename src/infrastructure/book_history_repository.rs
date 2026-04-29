use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::AuthorId,
            book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            change_set::ChangeSetId,
            history::{BookHistory, HistoryOperation},
            user::UserId,
        },
        error::DomainError,
        repository::book_history_repository::BookHistoryRepository,
    },
};

#[derive(sqlx::FromRow)]
struct BookHistoryRow {
    history_id: i64,
    change_set_id: Uuid,
    operation: String,
    book_id: Uuid,
    title: String,
    isbn: String,
    read: bool,
    owned: bool,
    priority: i32,
    format: String,
    store: String,
    book_created_at: OffsetDateTime,
    book_updated_at: OffsetDateTime,
    changed_at: OffsetDateTime,
    author_ids: Option<Vec<Uuid>>,
}

fn row_to_book_history(row: BookHistoryRow) -> Result<BookHistory, DomainError> {
    let operation = HistoryOperation::try_from(row.operation.as_str())
        .map_err(|e| DomainError::Unexpected(e))?;
    let book_id = BookId::new(row.book_id)?;
    let title = BookTitle::new(row.title)?;
    let isbn = Isbn::new(row.isbn)?;
    let priority = Priority::new(row.priority)?;
    let format = BookFormat::try_from(row.format.as_str())?;
    let store = BookStore::try_from(row.store.as_str())?;
    let author_ids: Vec<AuthorId> = row
        .author_ids
        .unwrap_or_default()
        .into_iter()
        .map(AuthorId::new)
        .collect();

    Ok(BookHistory {
        history_id: row.history_id,
        change_set_id: ChangeSetId::from(row.change_set_id),
        operation,
        book_id,
        title,
        author_ids,
        isbn,
        read: ReadFlag::new(row.read),
        owned: OwnedFlag::new(row.owned),
        priority,
        format,
        store,
        book_created_at: row.book_created_at,
        book_updated_at: row.book_updated_at,
        changed_at: row.changed_at,
    })
}

#[derive(Debug, Clone)]
pub struct PgBookHistoryRepository {
    pool: PgPool,
}

impl PgBookHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookHistoryRepository for PgBookHistoryRepository {
    async fn find_by_book(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Vec<BookHistory>, DomainError> {
        let rows: Vec<BookHistoryRow> = sqlx::query_as(
            "SELECT
                bh.history_id,
                bh.change_set_id,
                bh.operation,
                bh.book_id,
                bh.title,
                bh.isbn,
                bh.read,
                bh.owned,
                bh.priority,
                bh.format,
                bh.store,
                bh.book_created_at,
                bh.book_updated_at,
                bh.changed_at,
                array_agg(bha.author_id) FILTER (WHERE bha.author_id IS NOT NULL) AS author_ids
            FROM book_history bh
            LEFT JOIN book_history_author bha ON bh.history_id = bha.history_id
            WHERE bh.user_id = $1 AND bh.book_id = $2
            GROUP BY bh.history_id
            ORDER BY bh.changed_at DESC",
        )
        .bind(user_id.as_str())
        .bind(book_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_book_history).collect()
    }

    async fn find_by_history_id(
        &self,
        user_id: &UserId,
        history_id: i64,
    ) -> Result<Option<BookHistory>, DomainError> {
        let row: Option<BookHistoryRow> = sqlx::query_as(
            "SELECT
                bh.history_id,
                bh.change_set_id,
                bh.operation,
                bh.book_id,
                bh.title,
                bh.isbn,
                bh.read,
                bh.owned,
                bh.priority,
                bh.format,
                bh.store,
                bh.book_created_at,
                bh.book_updated_at,
                bh.changed_at,
                array_agg(bha.author_id) FILTER (WHERE bha.author_id IS NOT NULL) AS author_ids
            FROM book_history bh
            LEFT JOIN book_history_author bha ON bh.history_id = bha.history_id
            WHERE bh.user_id = $1 AND bh.history_id = $2
            GROUP BY bh.history_id",
        )
        .bind(user_id.as_str())
        .bind(history_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_book_history).transpose()
    }
}
