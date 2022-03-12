use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct User {
    id: Uuid,
    sub: String,
}

struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    async fn find_by_id(&self, id: Uuid) -> User {
        let mut conn = self.pool.acquire().await.unwrap();
        InternalUserRepository::find_by_id(id, &mut conn).await
    }
}

struct InternalUserRepository {}

impl InternalUserRepository {
    async fn find_by_id(id: Uuid, conn: &mut PgConnection) -> User {
        let user: User = sqlx::query_as("SELECT * FROM bookshelf_user WHERE id = ?")
            .bind(id)
            .fetch_one(conn)
            .await
            .unwrap();
        user
    }
}
