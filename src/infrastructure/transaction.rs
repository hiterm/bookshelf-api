use async_trait::async_trait;
use sqlx::{PgConnection, PgPool, Postgres};
use uuid::Uuid;

use crate::domain::{
    entity::{
        event::EventSetOperation,
        operation::{NewOperation, OperationId},
        user::UserId,
    },
    error::DomainError,
    repository::transaction::{TransactionEventSet, TransactionManager, TransactionOperation},
};

/// A PostgreSQL transaction carrying the Operation context generated when it
/// was opened. The legacy EventSet id remains available only while Event
/// writers coexist with the new history model.
pub struct PgTransaction {
    tx: sqlx::Transaction<'static, Postgres>,
    operation_id: OperationId,
    event_set_id: Uuid,
    user_id: UserId,
    revision_number: Option<i32>,
}

impl TransactionEventSet for PgTransaction {
    fn event_set_id(&self) -> Uuid {
        self.event_set_id
    }
}

impl TransactionOperation for PgTransaction {
    fn operation_id(&self) -> OperationId {
        self.operation_id.clone()
    }

    fn revision_number(&self) -> Option<i32> {
        self.revision_number
    }
}

impl PgTransaction {
    pub fn event_set_id(&self) -> Uuid {
        <Self as TransactionEventSet>::event_set_id(self)
    }

    pub fn operation_id(&self) -> OperationId {
        <Self as TransactionOperation>::operation_id(self)
    }

    pub fn set_revision_number(&mut self, revision_number: i32) {
        self.revision_number = Some(revision_number);
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
        self.begin_operation(user_id, &operation.into()).await
    }

    async fn begin_operation(
        &self,
        user_id: &UserId,
        operation: &NewOperation,
    ) -> Result<Self::Transaction, DomainError> {
        let mut tx = self.pool.begin().await?;

        let operation_id = OperationId::new();
        let detail = operation
            .detail
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| DomainError::Unexpected(error.to_string()))?;
        sqlx::query(
            "INSERT INTO operation (id, user_id, type, detail, undo_of_operation_id) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(operation_id.to_uuid())
        .bind(user_id.as_str())
        .bind(operation.operation_type.as_str())
        .bind(detail)
        .bind(
            operation
                .undo_of_operation_id
                .as_ref()
                .map(OperationId::to_uuid),
        )
        .execute(&mut *tx)
        .await?;

        // Legacy Event writers still need an EventSet until PR 3 cleanup.
        let event_set_id = Uuid::new_v4();
        sqlx::query("INSERT INTO event_set (id, user_id, operation) VALUES ($1, $2, $3)")
            .bind(event_set_id)
            .bind(user_id.as_str())
            .bind(operation.operation_type.as_str())
            .execute(&mut *tx)
            .await?;

        Ok(PgTransaction {
            tx,
            operation_id,
            event_set_id,
            user_id: user_id.clone(),
            revision_number: None,
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
                operation::{NewOperation, OperationDetail, OperationType},
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
    async fn rollback_removes_operation_and_event_set(pool: PgPool) -> anyhow::Result<()> {
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
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM operation WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(count, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn commit_keeps_operation_and_event_set(pool: PgPool) -> anyhow::Result<()> {
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
        let (operation_type,): (String,) =
            sqlx::query_as("SELECT type FROM operation WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(operation_type, "import_books");
        Ok(())
    }

    #[sqlx::test]
    async fn begin_operation_persists_typed_detail(pool: PgPool) -> anyhow::Result<()> {
        let user_id = user(&pool).await?;
        let manager = PgTransactionManager::new(pool.clone());
        let operation = NewOperation {
            operation_type: OperationType::ImportBooks,
            detail: Some(OperationDetail::ImportBooks { imported_count: 7 }),
            undo_of_operation_id: None,
        };
        let tx = manager.begin_operation(&user_id, &operation).await?;
        let operation_id = tx.operation_id();
        manager.commit(tx).await?;

        let (detail,): (serde_json::Value,) =
            sqlx::query_as("SELECT detail FROM operation WHERE id = $1")
                .bind(operation_id.to_uuid())
                .fetch_one(&pool)
                .await?;
        assert_eq!(
            detail,
            serde_json::json!({"type": "import_books", "imported_count": 7})
        );
        Ok(())
    }
}
