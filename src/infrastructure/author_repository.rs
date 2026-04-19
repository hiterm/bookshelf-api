use std::collections::HashMap;

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::PgPool;
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
        sqlx::query("INSERT INTO author (id, user_id, name) VALUES ($1, $2, $3)")
            .bind(author.id().to_uuid())
            .bind(user_id.as_str())
            .bind(author.name().as_str())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    async fn find_by_id(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Option<Author>, DomainError> {
        let row: Option<AuthorRow> =
            sqlx::query_as("SELECT * FROM author WHERE id = $1 AND user_id = $2")
                .bind(author_id.to_uuid())
                .bind(user_id.as_str())
                .fetch_optional(&self.pool)
                .await?;

        row.map(|row| -> Result<Author, DomainError> {
            let author_id: AuthorId = row.id.into();
            let author_name = AuthorName::new(row.name)?;
            Author::new(author_id, author_name)
        })
        .transpose()
    }

    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Author>, DomainError> {
        let authors: Result<Vec<Author>, DomainError> =
            sqlx::query_as("SELECT * FROM author WHERE user_id = $1 ORDER BY name ASC")
                .bind(user_id.as_str())
                .fetch(&self.pool)
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

    async fn update(&self, user_id: &UserId, author: &Author) -> Result<(), DomainError> {
        let result = sqlx::query(
            "UPDATE author SET name = $1, updated_at = now() WHERE id = $2 AND user_id = $3",
        )
        .bind(author.name().as_str())
        .bind(author.id().to_uuid())
        .bind(user_id.as_str())
        .execute(&self.pool)
        .await?;

        match result.rows_affected() {
            0 => Err(DomainError::NotFound {
                entity_type: "author",
                entity_id: author.id().to_string(),
                user_id: user_id.to_owned().into_string(),
            }),
            1 => Ok(()),
            _ => Err(DomainError::Unexpected(String::from(
                "rows_affected is greater than 1.",
            ))),
        }
    }

    async fn delete(&self, user_id: &UserId, author_id: &AuthorId) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query("DELETE FROM author WHERE id = $1 AND user_id = $2")
            .bind(author_id.to_uuid())
            .bind(user_id.as_str())
            .execute(&mut *tx)
            .await?;

        match result.rows_affected() {
            0 => {
                return Err(DomainError::NotFound {
                    entity_type: "author",
                    entity_id: author_id.to_string(),
                    user_id: user_id.to_owned().into_string(),
                });
            }
            1 => {}
            _ => {
                return Err(DomainError::Unexpected(String::from(
                    "rows_affected is greater than 1.",
                )));
            }
        }

        sqlx::query("DELETE FROM book_author WHERE user_id = $1 AND author_id = $2")
            .bind(user_id.as_str())
            .bind(author_id.to_uuid())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn find_by_ids_as_hash_map(
        &self,
        user_id: &UserId,
        author_ids: &[AuthorId],
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
        .fetch(&self.pool)
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
        common::types::{BookFormat, BookStore},
        domain::{
            entity::{
                book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                user::User,
            },
            error::DomainError,
            repository::{book_repository::BookRepository, user_repository::UserRepository},
        },
        infrastructure::{book_repository::PgBookRepository, user_repository::PgUserRepository},
    };
    use time::{
        PrimitiveDateTime,
        macros::{date, time},
    };

    use super::*;

    #[sqlx::test]
    async fn create_and_find_by_id(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

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

        let user_id = prepare_user(&user_repository, "user1").await?;

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

        let user_id = prepare_user(&user_repository, "user1").await?;

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

    #[sqlx::test]
    async fn find_by_id_does_not_return_other_users_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        author_repository.create(&user1_id, &author).await?;

        let result = author_repository.find_by_id(&user2_id, &author_id).await?;
        assert_eq!(result, None);

        Ok(())
    }

    #[sqlx::test]
    async fn find_all_does_not_return_other_users_authors(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new(author_id, AuthorName::new("author1".to_string())?)?;
        author_repository.create(&user1_id, &author).await?;

        let result = author_repository.find_all(&user2_id).await?;
        assert_eq!(result.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_ids_as_hash_map_does_not_return_other_users_authors(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        author_repository.create(&user1_id, &author).await?;

        let result = author_repository
            .find_by_ids_as_hash_map(&user2_id, &[author_id])
            .await?;
        assert_eq!(result.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn update_changes_name(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new(author_id.clone(), AuthorName::new("original".to_string())?)?;
        author_repository.create(&user_id, &author).await?;

        let updated = Author::new(author_id.clone(), AuthorName::new("updated".to_string())?)?;
        author_repository.update(&user_id, &updated).await?;

        let actual = author_repository.find_by_id(&user_id, &author_id).await?;
        assert_eq!(
            actual.map(|a| a.name().as_str().to_string()),
            Some("updated".to_string())
        );

        Ok(())
    }

    #[sqlx::test]
    async fn update_returns_not_found_for_nonexistent_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new(author_id, AuthorName::new("name".to_string())?)?;

        let result = author_repository.update(&user_id, &author).await;
        assert!(matches!(result, Err(DomainError::NotFound { .. })));

        Ok(())
    }

    #[sqlx::test]
    async fn update_returns_not_found_for_other_users_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new(author_id.clone(), AuthorName::new("name".to_string())?)?;
        author_repository.create(&user1_id, &author).await?;

        let updated = Author::new(author_id, AuthorName::new("hacked".to_string())?)?;
        let result = author_repository.update(&user2_id, &updated).await;
        assert!(matches!(result, Err(DomainError::NotFound { .. })));

        Ok(())
    }

    #[sqlx::test]
    async fn delete_removes_author_and_book_author_rows(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author = Author::new(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        author_repository.create(&user_id, &author).await?;

        let book = make_book("675bc8d9-3155-42fb-87b0-0a82cb162848", &[author_id.clone()])?;
        book_repository.create(&user_id, &book).await?;

        author_repository.delete(&user_id, &author_id).await?;

        let found = author_repository.find_by_id(&user_id, &author_id).await?;
        assert_eq!(found, None);

        // book_author row must be gone — find_by_id returns the book with no authors
        let book_after = book_repository.find_by_id(&user_id, book.id()).await?;
        assert!(
            book_after
                .map(|b| b.author_ids().is_empty())
                .unwrap_or(true)
        );

        Ok(())
    }

    #[sqlx::test]
    async fn delete_returns_not_found_for_nonexistent_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository, "user1").await?;

        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let result = author_repository.delete(&user_id, &author_id).await;
        assert!(matches!(result, Err(DomainError::NotFound { .. })));

        Ok(())
    }

    #[sqlx::test]
    async fn delete_does_not_touch_other_users_book_author(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repository, "user1").await?;
        let user2_id = prepare_user(&user_repository, "user2").await?;

        // Both users have the same author UUID — allowed by composite PK (id, user_id)
        let author_id = AuthorId::try_from("e324be11-5b77-4ba6-8423-9f27e2d228f1")?;
        let author1 = Author::new(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        let author2 = Author::new(author_id.clone(), AuthorName::new("author1".to_string())?)?;
        author_repository.create(&user1_id, &author1).await?;
        author_repository.create(&user2_id, &author2).await?;

        let book1 = make_book("675bc8d9-3155-42fb-87b0-0a82cb162848", &[author_id.clone()])?;
        book_repository.create(&user1_id, &book1).await?;
        let book2 = make_book("675bc8d9-3155-42fb-87b0-0a82cb162848", &[author_id.clone()])?;
        book_repository.create(&user2_id, &book2).await?;

        // user2 deletes their copy of the author — must not affect user1's book_author
        author_repository.delete(&user2_id, &author_id).await?;

        let user1_book = book_repository
            .find_by_id(&user1_id, book1.id())
            .await?
            .expect("user1's book must still exist");
        assert!(
            user1_book.author_ids().contains(&author_id),
            "user1's book_author row must be intact"
        );

        Ok(())
    }

    fn make_book(book_id_str: &str, author_ids: &[AuthorId]) -> Result<Book, DomainError> {
        let book_id = BookId::try_from(book_id_str)?;
        let title = BookTitle::new("title1".to_owned())?;
        let isbn = Isbn::new("1111111111116".to_owned())?;
        let read = ReadFlag::new(false);
        let owned = OwnedFlag::new(false);
        let priority = Priority::new(50)?;
        let format = BookFormat::EBook;
        let store = BookStore::Kindle;
        let created_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        let updated_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        Book::new(
            book_id,
            title,
            author_ids.to_vec(),
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            created_at,
            updated_at,
        )
    }

    async fn prepare_user(repository: &PgUserRepository, id: &str) -> Result<UserId, DomainError> {
        let user_id = UserId::new(String::from(id))?;
        let user = User::new(user_id.clone());
        repository.create(&user).await?;

        Ok(user_id)
    }
}
