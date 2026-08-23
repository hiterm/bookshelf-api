use async_trait::async_trait;
use sqlx::{PgConnection, PgPool, Postgres};
use uuid::Uuid;

use crate::domain::{
    entity::{event::EventSetOperation, user::UserId},
    error::DomainError,
    repository::transaction::{TransactionEventSet, TransactionManager},
};

/// A PostgreSQL transaction carrying the `event_set` id generated when the
/// transaction was opened. Repositories read `event_set_id()` instead of
/// generating their own UUID, and use `as_mut()` to run queries on the
/// transaction connection. The transaction rolls back if dropped without
/// `commit`.
pub struct PgTransaction {
    tx: sqlx::Transaction<'static, Postgres>,
    event_set_id: Uuid,
    user_id: UserId,
}

impl TransactionEventSet for PgTransaction {
    fn event_set_id(&self) -> Uuid {
        self.event_set_id
    }
}

impl PgTransaction {
    pub fn event_set_id(&self) -> Uuid {
        <Self as TransactionEventSet>::event_set_id(self)
    }

    /// Returns the user passed to `begin`, which is the single source of
    /// truth for mutating repository operations in this transaction.
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    // Named `as_mut` to mirror the `&mut *tx` access the repositories used
    // before this refactor; implementing std::convert::AsMut is unnecessary
    // because callers only ever need this concrete &mut PgConnection.
    #[allow(clippy::should_implement_trait)]
    pub fn as_mut(&mut self) -> &mut PgConnection {
        &mut self.tx
    }

    pub async fn commit(self) -> Result<(), DomainError> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), DomainError> {
        self.tx.rollback().await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PgTransactionManager {
    pool: PgPool,
}

impl PgTransactionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionManager for PgTransactionManager {
    type Transaction = PgTransaction;

    async fn begin(
        &self,
        user_id: &UserId,
        operation: EventSetOperation,
    ) -> Result<Self::Transaction, DomainError> {
        let mut tx = self.pool.begin().await?;

        let event_set_id = Uuid::new_v4();
        sqlx::query("INSERT INTO event_set (id, user_id, operation) VALUES ($1, $2, $3)")
            .bind(event_set_id)
            .bind(user_id.as_str())
            .bind(operation.as_str())
            .execute(&mut *tx)
            .await?;

        Ok(PgTransaction {
            tx,
            event_set_id,
            user_id: user_id.clone(),
        })
    }

    async fn commit(&self, tx: Self::Transaction) -> Result<(), DomainError> {
        tx.commit().await
    }

    async fn rollback(&self, tx: Self::Transaction) -> Result<(), DomainError> {
        tx.rollback().await
    }
}

#[cfg(all(test, feature = "test-with-database"))]
mod tests {
    use sqlx::PgPool;

    use crate::{
        domain::{
            entity::{
                event::EventSetOperation,
                user::{User, UserId},
            },
            repository::{transaction::TransactionManager, user_repository::UserRepository},
        },
        infrastructure::{transaction::PgTransactionManager, user_repository::PgUserRepository},
    };

    async fn user(pool: &PgPool) -> anyhow::Result<UserId> {
        let id = UserId::new("transaction-user".to_string())?;
        PgUserRepository::new(pool.clone())
            .create(&User::new(id.clone()))
            .await?;
        Ok(id)
    }

    #[sqlx::test]
    async fn rollback_removes_event_set(pool: PgPool) -> anyhow::Result<()> {
        let user_id = user(&pool).await?;
        let manager = PgTransactionManager::new(pool.clone());
        let tx = manager
            .begin(&user_id, EventSetOperation::ImportBooks)
            .await?;
        manager.rollback(tx).await?;

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM event_set WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(count, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn commit_keeps_event_set(pool: PgPool) -> anyhow::Result<()> {
        let user_id = user(&pool).await?;
        let manager = PgTransactionManager::new(pool.clone());
        let tx = manager
            .begin(&user_id, EventSetOperation::ImportBooks)
            .await?;
        manager.commit(tx).await?;

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM event_set WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(count, 1);
        Ok(())
    }
}
