use async_trait::async_trait;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::{
    entity::user::User, error::domain_error::DomainError,
    repository::user_repository::UserRepository,
};

// TODO unwrapをやめる

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    sub: String,
}

struct PgUserRepository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        let mut conn = self.pool.acquire().await.unwrap();
        let result = InternalUserRepository::create(user, &mut conn).await?;
        Ok(result)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let mut conn = self.pool.acquire().await.unwrap();
        let user = InternalUserRepository::find_by_id(id, &mut conn).await?;
        Ok(user)
    }
}

struct InternalUserRepository {}

impl InternalUserRepository {
    async fn create(user: &User, conn: &mut PgConnection) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO bookshelf_user (id, sub) VALUES ($1, $2)")
            .bind(user.id())
            .bind(user.sub())
            .execute(conn)
            .await
            .unwrap();
        Ok(())
    }

    async fn find_by_id(id: Uuid, conn: &mut PgConnection) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM bookshelf_user WHERE id = $1")
            .bind(id)
            .fetch_optional(conn)
            .await
            .unwrap();

        Ok(row.map(|row| User::new(row.id, row.sub)))
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

        let id = Uuid::parse_str("e112995d-3e3b-4d32-8c25-7ce9451ab18b").unwrap();
        let user = User::new(id, String::from("foo"));

        let fetched_user = InternalUserRepository::find_by_id(id, &mut tx)
            .await
            .unwrap();
        assert!(fetched_user.is_none());

        InternalUserRepository::create(&user, &mut tx)
            .await
            .unwrap();

        let fetched_user = InternalUserRepository::find_by_id(id, &mut tx)
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
