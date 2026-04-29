use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        author::AuthorId,
        change_set::ChangeSetId,
        history::{AuthorHistory, HistoryOperation},
        user::UserId,
    },
    error::DomainError,
    repository::author_history_repository::AuthorHistoryRepository,
};

#[derive(sqlx::FromRow)]
struct AuthorHistoryRow {
    history_id: i64,
    change_set_id: Uuid,
    operation: String,
    author_id: Uuid,
    name: String,
    yomi: String,
    author_created_at: OffsetDateTime,
    author_updated_at: OffsetDateTime,
    changed_at: OffsetDateTime,
}

fn row_to_author_history(row: AuthorHistoryRow) -> Result<AuthorHistory, DomainError> {
    let operation =
        HistoryOperation::try_from(row.operation.as_str()).map_err(DomainError::Unexpected)?;

    Ok(AuthorHistory {
        history_id: row.history_id,
        change_set_id: ChangeSetId::from(row.change_set_id),
        operation,
        author_id: AuthorId::new(row.author_id),
        name: row.name,
        yomi: row.yomi,
        author_created_at: row.author_created_at,
        author_updated_at: row.author_updated_at,
        changed_at: row.changed_at,
    })
}

#[derive(Debug, Clone)]
pub struct PgAuthorHistoryRepository {
    pool: PgPool,
}

impl PgAuthorHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorHistoryRepository for PgAuthorHistoryRepository {
    async fn find_by_author(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Vec<AuthorHistory>, DomainError> {
        let rows: Vec<AuthorHistoryRow> = sqlx::query_as(
            "SELECT history_id, change_set_id, operation, author_id, name, yomi,
                    author_created_at, author_updated_at, changed_at
             FROM author_history
             WHERE user_id = $1 AND author_id = $2
             ORDER BY changed_at DESC",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_author_history).collect()
    }

    async fn find_by_history_id(
        &self,
        user_id: &UserId,
        history_id: i64,
    ) -> Result<Option<AuthorHistory>, DomainError> {
        let row: Option<AuthorHistoryRow> = sqlx::query_as(
            "SELECT history_id, change_set_id, operation, author_id, name, yomi,
                    author_created_at, author_updated_at, changed_at
             FROM author_history
             WHERE user_id = $1 AND history_id = $2",
        )
        .bind(user_id.as_str())
        .bind(history_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_author_history).transpose()
    }
}
