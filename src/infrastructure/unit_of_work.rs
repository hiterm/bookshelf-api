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
    async fn test_begin_commit_persists_changes(pool: PgPool) -> anyhow::Result<()> {
        let mut uow = PgUnitOfWork::begin(&pool).await?;

        sqlx::query("CREATE TEMP TABLE uow_test (id INT)")
            .execute(&mut **uow.tx())
            .await?;

        sqlx::query("INSERT INTO uow_test (id) VALUES (1)")
            .execute(&mut **uow.tx())
            .await?;

        uow.commit().await?;

        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM uow_test")
            .fetch_one(&pool)
            .await?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_begin_rollback_discards_changes(pool: PgPool) -> anyhow::Result<()> {
        let mut uow = PgUnitOfWork::begin(&pool).await?;

        sqlx::query("CREATE TEMP TABLE uow_test (id INT)")
            .execute(&mut **uow.tx())
            .await?;

        sqlx::query("INSERT INTO uow_test (id) VALUES (1)")
            .execute(&mut **uow.tx())
            .await?;

        uow.rollback().await?;

        // After rollback, the temp table (and its data) should not be visible
        let result = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM uow_test")
            .fetch_one(&pool)
            .await;
        assert!(result.is_err());

        Ok(())
    }
}
