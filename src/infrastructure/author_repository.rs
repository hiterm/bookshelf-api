use async_trait::async_trait;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::domain::{
    entity::{
        author::{Author, AuthorId, AuthorName},
        user::UserId,
    },
    error::DomainError,
    repository::author_repository::AuthorRepository,
};

#[derive(sqlx::FromRow)]
struct AuthorRow {
    id: Uuid,
    // TODO: remove attribute
    #[allow(unused)]
    user_id: String,
    name: String,
}

pub struct PgAuthorRepository {
    pub pool: PgPool,
}

#[async_trait]
impl AuthorRepository for PgAuthorRepository {
    // TODO: remove attribute
    #[allow(unused)]
    async fn create(&self, user_id: &UserId, author: &Author) -> Result<(), DomainError> {
        todo!()
    }
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        InternalAuthorRepository::find_by_id(user_id, author_id, &mut conn).await
    }
}

struct InternalAuthorRepository {}

impl InternalAuthorRepository {
    // TODO: remove attribute
    #[allow(unused)]
    async fn create(
        user_id: &UserId,
        author: &Author,
        conn: &mut PgConnection,
    ) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO author (id, user_id, name) VALUES ($1, $2, $3)")
            .bind(author.id().id())
            .bind(user_id.id())
            .bind(author.name().name())
            .execute(conn)
            .await?;
        Ok(())
    }

    async fn find_by_id(
        user_id: &UserId,
        author_id: &AuthorId,
        conn: &mut PgConnection,
    ) -> Result<Option<Author>, DomainError> {
        let row: Option<AuthorRow> =
            sqlx::query_as("SELECT * FROM author WHERE id = $1 AND user_id = $2")
                .bind(author_id.id())
                .bind(user_id.id())
                .fetch_optional(conn)
                .await?;

        row.map(|row| -> Result<Author, DomainError> {
            let author_id: AuthorId = row.id.into();
            let author_name = AuthorName::new(row.name)?;
            Author::new(author_id, author_name)
        })
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        domain::entity::user::User, infrastructure::user_repository::InternalUserRepository,
    };

    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn create_and_find() -> anyhow::Result<()> {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await?;
        let mut tx = pool.begin().await?;

        let user_id = UserId::new(String::from("user1"))?;
        let user = User::new(user_id.clone());
        let author_id = AuthorId::new("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author = Author::new(author_id.clone(), author_name)?;

        InternalUserRepository::create(&user, &mut tx).await?;
        InternalAuthorRepository::create(&user_id, &author, &mut tx).await?;

        let actual = InternalAuthorRepository::find_by_id(&user_id, &author_id, &mut tx).await?;
        assert_eq!(actual, Some(author));

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
