use async_trait::async_trait;
use sqlx::PgPool;

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
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO bookshelf_user (id) VALUES ($1)")
            .bind(user.id.as_str())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM bookshelf_user WHERE id = $1")
            .bind(id.as_str())
            .fetch_optional(&self.pool)
            .await?;

        let id = row.map(|row| UserId::new(row.id)).transpose()?;
        Ok(id.map(User::new))
    }
}

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_user_repository(pool: PgPool) -> anyhow::Result<()> {
        dotenv::dotenv().ok();

        let repository = PgUserRepository::new(pool);

        // let mut tx = pool.begin().await?;

        let id = UserId::new(String::from("foo"))?;
        let user = User::new(id.clone());

        let fetched_user = repository.find_by_id(&id).await?;
        assert!(fetched_user.is_none());

        repository.create(&user).await?;

        let fetched_user = repository.find_by_id(&id).await?;
        assert_eq!(fetched_user, Some(user));

        Ok(())
    }
}
