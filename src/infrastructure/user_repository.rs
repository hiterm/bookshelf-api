use async_trait::async_trait;
use sqlx::{PgConnection, PgPool};

use crate::domain::{
    entity::user::User, error::domain_error::DomainError,
    repository::user_repository::UserRepository,
};

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
}

struct PgUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        let mut conn = self.pool.acquire().await?;
        let result = InternalUserRepository::create(user, &mut conn).await?;
        Ok(result)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        let user = InternalUserRepository::find_by_id(id, &mut conn).await?;
        Ok(user)
    }
}

struct InternalUserRepository {}

impl InternalUserRepository {
    async fn create(user: &User, conn: &mut PgConnection) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO bookshelf_user (id) VALUES ($1)")
            .bind(user.id())
            .execute(conn)
            .await?;
        Ok(())
    }

    async fn find_by_id(id: &str, conn: &mut PgConnection) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM bookshelf_user WHERE id = $1")
            .bind(id)
            .fetch_optional(conn)
            .await?;

        Ok(row.map(|row| User::new(row.id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn test_user_repository() {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();

        let id = String::from("foo");
        let user = User::new(id.clone());

        let fetched_user = InternalUserRepository::find_by_id(&id, &mut tx)
            .await
            .unwrap();
        assert!(fetched_user.is_none());

        InternalUserRepository::create(&user, &mut tx)
            .await
            .unwrap();

        let fetched_user = InternalUserRepository::find_by_id(&id, &mut tx)
            .await
            .unwrap();
        assert_eq!(fetched_user, Some(user));

        tx.rollback().await.unwrap();
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
