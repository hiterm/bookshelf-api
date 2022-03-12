use sqlx::{PgPool, PgConnection};
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
    async fn find_by_id(id: Uuid, conn: &mut PgConnection) -> User {
        let user: User = sqlx::query_as("SELECT * FROM bookshelf_user WHERE id = ?")
            .bind(id)
            .fetch_one(conn)
            .await
            .unwrap();
        user
    }
}
