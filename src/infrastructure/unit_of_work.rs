use sqlx::{PgPool, Postgres, Transaction};

use crate::domain::error::DomainError;

pub struct PgUnitOfWork {
    tx: Transaction<'static, Postgres>,
}

impl PgUnitOfWork {
    pub async fn begin(pool: &PgPool) -> Result<Self, DomainError> {
        let tx = pool.begin().await?;
        Ok(Self { tx })
    }

    pub fn tx(&mut self) -> &mut Transaction<'static, Postgres> {
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

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_begin_commit(pool: PgPool) -> anyhow::Result<()> {
        let mut uow = PgUnitOfWork::begin(&pool).await?;

        // Verify the transaction handle is usable
        let (count,): (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&mut **uow.tx())
            .await?;
        assert_eq!(count, 1);

        uow.commit().await?;

        Ok(())
    }

    #[sqlx::test]
    async fn test_begin_rollback(pool: PgPool) -> anyhow::Result<()> {
        let mut uow = PgUnitOfWork::begin(&pool).await?;

        // Verify the transaction handle is usable
        let (count,): (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&mut **uow.tx())
            .await?;
        assert_eq!(count, 1);

        uow.rollback().await?;

        Ok(())
    }
}
