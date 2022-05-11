use std::collections::HashMap;

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
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
    name: String,
}

#[derive(Debug, Clone)]
pub struct PgAuthorRepository {
    pool: PgPool,
}

impl PgAuthorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorRepository for PgAuthorRepository {
    async fn create(&self, user_id: &UserId, author: &Author) -> Result<(), DomainError> {
        let mut conn = self.pool.acquire().await?;
        InternalAuthorRepository::create(user_id, author, &mut conn).await?;
        Ok(())
    }
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        InternalAuthorRepository::find_by_id(user_id, author_id, &mut conn).await
    }

    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Author>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        InternalAuthorRepository::find_all(user_id, &mut conn).await
    }

    async fn find_by_ids_as_hash_map(
        &self,
        user_id: &UserId,
        author_ids: &[AuthorId],
    ) -> Result<HashMap<AuthorId, Author>, DomainError> {
        todo!()
    }
}

pub(in crate::infrastructure) struct InternalAuthorRepository {}

impl InternalAuthorRepository {
    pub(in crate::infrastructure) async fn create(
        user_id: &UserId,
        author: &Author,
        conn: &mut PgConnection,
    ) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO author (id, user_id, name) VALUES ($1, $2, $3)")
            .bind(author.id().to_uuid())
            .bind(user_id.as_str())
            .bind(author.name().as_str())
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
                .bind(author_id.to_uuid())
                .bind(user_id.as_str())
                .fetch_optional(conn)
                .await?;

        row.map(|row| -> Result<Author, DomainError> {
            let author_id: AuthorId = row.id.into();
            let author_name = AuthorName::new(row.name)?;
            Author::new(author_id, author_name)
        })
        .transpose()
    }

    async fn find_all(
        user_id: &UserId,
        conn: &mut PgConnection,
    ) -> Result<Vec<Author>, DomainError> {
        let authors: Result<Vec<Author>, DomainError> =
            sqlx::query_as("SELECT * FROM author WHERE user_id = $1 ORDER BY name ASC")
                .bind(user_id.as_str())
                .fetch(conn)
                .map(
                    |row: Result<AuthorRow, sqlx::Error>| -> Result<Author, DomainError> {
                        let row = row?;
                        let author_id = AuthorId::new(row.id);
                        let author_name = AuthorName::new(row.name)?;
                        let author = Author::new(author_id, author_name)?;
                        Ok(author)
                    },
                )
                .try_collect()
                .await;

        Ok(authors?)
    }

    async fn find_by_ids_as_hash_map(
        user_id: &UserId,
        author_ids: &[AuthorId],
        conn: &mut PgConnection,
    ) -> Result<HashMap<AuthorId, Author>, DomainError> {
        let author_ids: Vec<Uuid> = author_ids
            .iter()
            .map(|author_id| author_id.to_uuid())
            .collect();

        let authors_map: HashMap<AuthorId, Author> = sqlx::query_as(
            "SELECT * FROM author WHERE user_id = $1 AND id = ANY($2) ORDER BY name ASC",
        )
        .bind(user_id.as_str())
        .bind(author_ids)
        .fetch(conn)
        .map(
            |row: Result<AuthorRow, sqlx::Error>| -> Result<(AuthorId, Author), DomainError> {
                let row = row?;
                let author_id = AuthorId::new(row.id);
                let author_name = AuthorName::new(row.name)?;
                let author = Author::new(author_id.clone(), author_name)?;
                Ok((author_id, author))
            },
        )
        .try_collect()
        .await?;

        Ok(authors_map)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        domain::entity::user::User, infrastructure::user_repository::InternalUserRepository,
    };

    use super::*;
    use serde::de::Expected;
    use sqlx::{postgres::PgPoolOptions, Postgres, Transaction};

    #[tokio::test]
    #[ignore] // Depends on PostgreSQL
    async fn create_and_find_by_id() -> anyhow::Result<()> {
        let mut tx = prepare_tx().await?;

        let user_id = prepare_user(&mut tx).await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author = Author::new(author_id.clone(), author_name)?;

        InternalAuthorRepository::create(&user_id, &author, &mut tx).await?;

        let actual = InternalAuthorRepository::find_by_id(&user_id, &author_id, &mut tx).await?;
        assert_eq!(actual, Some(author.clone()));

        tx.rollback().await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Depends on PostgreSQL
    async fn create_and_find_all() -> anyhow::Result<()> {
        let mut tx = prepare_tx().await?;

        let user_id = prepare_user(&mut tx).await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author1 = Author::new(author_id.clone(), author_name)?;

        let author_id = AuthorId::try_from("e9700384-6217-4152-88c0-7ba38aeee73a")?;
        let author_name = AuthorName::new(String::from("author2"))?;
        let author2 = Author::new(author_id.clone(), author_name)?;

        InternalAuthorRepository::create(&user_id, &author1, &mut tx).await?;
        InternalAuthorRepository::create(&user_id, &author2, &mut tx).await?;

        let all_authors = InternalAuthorRepository::find_all(&user_id, &mut tx).await?;
        assert_eq!(all_authors.len(), 2);
        assert_eq!(all_authors, vec![author1, author2]);

        tx.rollback().await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Depends on PostgreSQL
    async fn create_and_find_by_ids_as_hash_map() -> anyhow::Result<()> {
        let mut tx = prepare_tx().await?;

        let user_id = prepare_user(&mut tx).await?;

        let author_id1 = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author1 = Author::new(author_id1.clone(), author_name)?;

        let author_id2 = AuthorId::try_from("e9700384-6217-4152-88c0-7ba38aeee73a")?;
        let author_name = AuthorName::new(String::from("author2"))?;
        let author2 = Author::new(author_id2.clone(), author_name)?;

        InternalAuthorRepository::create(&user_id, &author1, &mut tx).await?;
        InternalAuthorRepository::create(&user_id, &author2, &mut tx).await?;

        let all_authors = InternalAuthorRepository::find_by_ids_as_hash_map(
            &user_id,
            &[author_id1.clone(), author_id2.clone()],
            &mut tx,
        )
        .await?;
        let mut expected = HashMap::new();
        expected.insert(author_id1, author1);
        expected.insert(author_id2, author2);

        assert_eq!(all_authors.len(), 2);
        assert_eq!(all_authors, expected);

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

    async fn prepare_tx() -> Result<Transaction<'static, Postgres>, DomainError> {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await?;

        Ok(pool.begin().await?)
    }

    async fn prepare_user(tx: &mut PgConnection) -> Result<UserId, DomainError> {
        let user_id = UserId::new(String::from("user1"))?;
        let user = User::new(user_id.clone());
        InternalUserRepository::create(&user, tx).await?;

        Ok(user_id)
    }
}
