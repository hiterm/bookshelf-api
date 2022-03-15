use async_trait::async_trait;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::{
    entity::{author::Author, user::User},
    error::domain_error::DomainError,
    repository::author_repository::AuthorRepository,
};

#[derive(sqlx::FromRow)]
struct AuthorRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
}

struct PgAuthorRepository {
    pool: PgPool,
}

#[async_trait]
impl AuthorRepository for PgAuthorRepository {
    async fn create(&self, user: User, author: &Author) -> Result<(), DomainError> {
        todo!()
    }
    async fn find_by_id(&self, user: User, author_id: Uuid) -> Result<Option<Author>, DomainError> {
        todo!()
    }
}

struct InternalAuthorRepository {}

impl InternalAuthorRepository {
    async fn create(
        user: User,
        author: &Author,
        conn: &mut PgConnection,
    ) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO author (id, user_id, name) VALUES ($1, $2, $3)")
            .bind(author.id())
            .bind(user.id())
            .bind(author.name())
            .execute(conn)
            .await?;
        Ok(())
    }

    async fn find_by_id(
        user: User,
        author_id: Uuid,
        conn: &mut PgConnection,
    ) -> Result<Option<Author>, DomainError> {
        let row: Option<AuthorRow> = sqlx::query_as("SELECT * FROM author WHERE id = $1 AND user_id = $2")
            .bind(author_id)
            .bind(user.id())
            .fetch_optional(conn)
            .await?;

        Ok(row.map(|row| Author::new(row.id, row.name)))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn create_and_find() {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();

        let user = User::new(String::from("user1"));
        let author_id = Uuid::parse_str("e324be11-5b77-4ba6-8423-9f27e2d228f1").unwrap();
        let author = Author::new(author_id, String::from("author1"));
        InternalAuthorRepository::create(user, &author, &mut tx).await.unwrap();
        // let actual = InternalAuthorRepository::find_by_id(author_id, &mut tx).await;

        tx.rollback().await.unwrap();

        // assert_eq!(actual.len(), 0);
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
