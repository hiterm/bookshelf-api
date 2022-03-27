use async_trait::async_trait;
use sqlx::{PgConnection, PgPool};

use crate::domain::{
    entity::user::{User, UserId},
    error::DomainError,
    repository::user_repository::UserRepository,
};

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
}

#[derive(Debug, Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        let mut conn = self.pool.acquire().await?;
        let result = InternalUserRepository::create(user, &mut conn).await?;
        Ok(result)
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        let user = InternalUserRepository::find_by_id(id, &mut conn).await?;
        Ok(user)
    }
}

pub(in crate::infrastructure) struct InternalUserRepository {}

impl InternalUserRepository {
    pub(in crate::infrastructure) async fn create(
        user: &User,
        conn: &mut PgConnection,
    ) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO bookshelf_user (id) VALUES ($1)")
            .bind(user.id.id.as_str())
            .execute(conn)
            .await?;
        Ok(())
    }

    async fn find_by_id(id: &UserId, conn: &mut PgConnection) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM bookshelf_user WHERE id = $1")
            .bind(id.id.as_str())
            .fetch_optional(conn)
            .await?;

        let id = row.map(|row| UserId::new(row.id)).transpose()?;
        Ok(id.map(|id| User::new(id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn test_user_repository() -> anyhow::Result<()> {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await?;
        let mut tx = pool.begin().await?;

        let id = UserId::new(String::from("foo"))?;
        let user = User::new(id.clone());

        let fetched_user = InternalUserRepository::find_by_id(&id, &mut tx).await?;
        assert!(fetched_user.is_none());

        InternalUserRepository::create(&user, &mut tx).await?;

        let fetched_user = InternalUserRepository::find_by_id(&id, &mut tx).await?;
        assert_eq!(fetched_user, Some(user));

        tx.rollback().await?;
        Ok(())
    }

    fn fetch_database_url() -> String {
        use std::env::VarError;

        match std::env::var("DATABASE_URL") {
            Ok(s) => s,
            Err(VarError::NotPresent) => panic!("Environment variable DATABASE_URL is required."),
            Err(VarError::NotUnicode(_)) => {
                panic!("Environment variable DATABASE_URL is not unicode.")
            }
        }
    }
}
