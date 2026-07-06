use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        author::AuthorId,
        event::{AuthorEvent, EventOperation},
        event_set::EventSetId,
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
        EventOperation::try_from(row.operation.as_str()).map_err(DomainError::Unexpected)?;

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

    async fn find_by_event_set(
        &self,
        user_id: &UserId,
        event_set_id: &EventSetId,
    ) -> Result<Vec<AuthorEvent>, DomainError> {
        let rows: Vec<AuthorEventRow> = sqlx::query_as(
            "SELECT event_id, event_set_id, operation, author_id, name, yomi,
                    author_created_at, author_updated_at, changed_at, extra
             FROM author_event
             WHERE user_id = $1 AND event_set_id = $2
             ORDER BY changed_at DESC",
        )
        .bind(user_id.as_str())
        .bind(event_set_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_author_event).collect()
    }
}

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {
    use crate::domain::{
        entity::{
            author::{Author, AuthorName},
            event::{EventOperation, EventSetOperation},
            user::User,
        },
        repository::{
            author_repository::AuthorRepository, transaction::TransactionManager,
            user_repository::UserRepository,
        },
    };
    use crate::infrastructure::{
        author_repository::PgAuthorRepository, transaction::PgTransactionManager,
        user_repository::PgUserRepository,
    };

    use super::*;

    async fn prepare_user(repository: &PgUserRepository, id: &str) -> Result<UserId, DomainError> {
        let user_id = UserId::new(id.to_string())?;
        repository.create(&User::new(user_id.clone())).await?;
        Ok(user_id)
    }

    async fn create_author(
        pool: &PgPool,
        author_repo: &PgAuthorRepository,
        user_id: &UserId,
        author: &Author,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::CreateAuthor).await?;
        author_repo.create(&mut tx, author).await?;
        tm.commit(tx).await
    }

    #[sqlx::test]
    async fn find_by_event_set_returns_events(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let event_repo = PgAuthorEventRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        create_author(
            &pool,
            &author_repo,
            &user_id,
            &Author::new(author_id, AuthorName::new("author1".to_owned())?)?,
        )
        .await?;

        let (event_set_id,): (Uuid,) =
            sqlx::query_as("SELECT event_set_id FROM author_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;

        let entries = event_repo
            .find_by_event_set(&user_id, &EventSetId::from(event_set_id))
            .await?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, EventOperation::Create);
        assert_eq!(entries[0].name.as_deref(), Some("author1"));

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_event_set_returns_empty_for_unknown_set(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let event_repo = PgAuthorEventRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let unknown = EventSetId::try_from("00000000-0000-0000-0000-000000000000")
            .map_err(DomainError::Unexpected)?;

        let entries = event_repo.find_by_event_set(&user_id, &unknown).await?;
        assert!(entries.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_event_set_is_user_scoped_returns_empty_for_other_user(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let event_repo = PgAuthorEventRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repo, "user1").await?;
        let user2_id = prepare_user(&user_repo, "user2").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        create_author(
            &pool,
            &author_repo,
            &user1_id,
            &Author::new(author_id, AuthorName::new("author1".to_owned())?)?,
        )
        .await?;

        let (event_set_id,): (Uuid,) =
            sqlx::query_as("SELECT event_set_id FROM author_event WHERE user_id = $1")
                .bind(user1_id.as_str())
                .fetch_one(&pool)
                .await?;

        // user2 must not see user1's event set events.
        let entries = event_repo
            .find_by_event_set(&user2_id, &EventSetId::from(event_set_id))
            .await?;
        assert!(entries.is_empty());

        Ok(())
    }
}
