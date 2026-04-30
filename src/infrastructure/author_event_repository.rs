use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        author::AuthorId,
        event_set::EventSetId,
        history::{AuthorEvent, HistoryOperation},
        user::UserId,
    },
    error::DomainError,
    repository::author_event_repository::AuthorEventRepository,
};

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

fn row_to_author_event(row: AuthorEventRow) -> Result<AuthorEvent, DomainError> {
    let operation =
        HistoryOperation::try_from(row.operation.as_str()).map_err(DomainError::Unexpected)?;

    Ok(AuthorEvent {
        event_id: row.event_id,
        event_set_id: EventSetId::from(row.event_set_id),
        operation,
        author_id: AuthorId::new(row.author_id),
        name: row.name,
        yomi: row.yomi,
        author_created_at: row.author_created_at,
        author_updated_at: row.author_updated_at,
        changed_at: row.changed_at,
        extra: row.extra,
    })
}

#[derive(Debug, Clone)]
pub struct PgAuthorEventRepository {
    pool: PgPool,
}

impl PgAuthorEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorEventRepository for PgAuthorEventRepository {
    async fn find_by_author(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Vec<AuthorEvent>, DomainError> {
        let rows: Vec<AuthorEventRow> = sqlx::query_as(
            "SELECT event_id, event_set_id, operation, author_id, name, yomi,
                    author_created_at, author_updated_at, changed_at, extra
             FROM author_event
             WHERE user_id = $1 AND author_id = $2
             ORDER BY changed_at DESC",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_author_event).collect()
    }

    async fn find_by_event_id(
        &self,
        user_id: &UserId,
        event_id: i64,
    ) -> Result<Option<AuthorEvent>, DomainError> {
        let row: Option<AuthorEventRow> = sqlx::query_as(
            "SELECT event_id, event_set_id, operation, author_id, name, yomi,
                    author_created_at, author_updated_at, changed_at, extra
             FROM author_event
             WHERE user_id = $1 AND event_id = $2",
        )
        .bind(user_id.as_str())
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_author_event).transpose()
    }
}
