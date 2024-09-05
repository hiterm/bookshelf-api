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
        let mut conn = self.pool.acquire().await?;
        InternalAuthorRepository::find_by_ids_as_hash_map(user_id, author_ids, &mut conn).await
    }
}

pub struct InternalAuthorRepository {}

impl InternalAuthorRepository {
    pub async fn create(
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

    pub async fn find_by_id(
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

        authors
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

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {

    use crate::{
        domain::{entity::user::User, repository::user_repository::UserRepository},
        infrastructure::user_repository::PgUserRepository,
    };

    use super::*;

    #[sqlx::test]
    async fn create_and_find_by_id(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository).await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author = Author::new(author_id.clone(), author_name)?;

        author_repository.create(&user_id, &author).await?;

        let actual = author_repository.find_by_id(&user_id, &author_id).await?;
        assert_eq!(actual, Some(author.clone()));

        Ok(())
    }

    #[sqlx::test]
    async fn create_and_find_all(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository).await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author1 = Author::new(author_id.clone(), author_name)?;

        let author_id = AuthorId::try_from("e9700384-6217-4152-88c0-7ba38aeee73a")?;
        let author_name = AuthorName::new(String::from("author2"))?;
        let author2 = Author::new(author_id.clone(), author_name)?;

        author_repository.create(&user_id, &author1).await?;
        author_repository.create(&user_id, &author2).await?;

        let all_authors = author_repository.find_all(&user_id).await?;
        assert_eq!(all_authors.len(), 2);
        assert_eq!(all_authors, vec![author1, author2]);

        Ok(())
    }

    #[sqlx::test]
    async fn create_and_find_by_ids_as_hash_map(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository).await?;

        let author_id1 = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author_name = AuthorName::new(String::from("author1"))?;
        let author1 = Author::new(author_id1.clone(), author_name)?;

        let author_id2 = AuthorId::try_from("e9700384-6217-4152-88c0-7ba38aeee73a")?;
        let author_name = AuthorName::new(String::from("author2"))?;
        let author2 = Author::new(author_id2.clone(), author_name)?;

        author_repository.create(&user_id, &author1).await?;
        author_repository.create(&user_id, &author2).await?;

        let all_authors = author_repository
            .find_by_ids_as_hash_map(&user_id, &[author_id1.clone(), author_id2.clone()])
            .await?;
        let mut expected = HashMap::new();
        expected.insert(author_id1, author1);
        expected.insert(author_id2, author2);

        assert_eq!(all_authors.len(), 2);
        assert_eq!(all_authors, expected);

        Ok(())
    }

    async fn prepare_user(repository: &PgUserRepository) -> Result<UserId, DomainError> {
        let user_id = UserId::new(String::from("user1"))?;
        let user = User::new(user_id.clone());
        repository.create(&user).await?;

        Ok(user_id)
    }
}
