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
        InternalUserRepository::create(user, &mut conn).await
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let mut conn = self.pool.acquire().await.unwrap();
        Ok(InternalUserRepository::find_by_id(id, &mut conn).await)
    }
}

struct InternalUserRepository {}

impl InternalUserRepository {
    async fn create(user: &User, conn: &mut PgConnection) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO user (id, sub) VALUES ($1, $2)")
            .bind(user.id())
            .bind(user.sub())
            .execute(conn)
            .await
            .unwrap();
        Ok(())
    }

    async fn find_by_id(id: Uuid, conn: &mut PgConnection) -> Option<User> {
        let row: Option<UserRow> = sqlx::query_as("SELECT * FROM bookshelf_user WHERE id = $1")
            .bind(id)
            .fetch_optional(conn)
            .await
            .unwrap();

        row.map(|row| User::new(row.id, row.sub))
    }
}
