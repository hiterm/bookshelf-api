use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{PgConnection, PgPool};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        book::{
            Book, BookFormat, BookId, BookStore, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag,
        },
        user::UserId,
    },
    error::DomainError,
    repository::book_repository::BookRepository,
};

#[derive(sqlx::FromRow)]
struct BookRow {
    id: Uuid,
    title: String,
    isbn: String,
    read: bool,
    owned: bool,
    priority: i32,
    format: String,
    store: String,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

struct PgBookRepository {
    pool: PgPool,
}

#[async_trait]
impl BookRepository for PgBookRepository {
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Book>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        InternalBookRepository::find_all(user_id, &mut conn).await
    }
}

struct InternalBookRepository {}

impl InternalBookRepository {
    async fn find_all(user_id: &UserId, conn: &mut PgConnection) -> Result<Vec<Book>, DomainError> {
        let books: Result<Vec<Book>, DomainError> =
            sqlx::query_as("SELECT * FROM book WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch(conn)
                .map(
                    |row: Result<BookRow, sqlx::Error>| -> Result<Book, DomainError> {
                        let row = row?;
                        let book_id = BookId::new(row.id)?;
                        let title = BookTitle::new(row.title)?;
                        let isbn = Isbn::new(row.isbn)?;
                        let read = ReadFlag::new(row.read);
                        let owned = OwnedFlag::new(row.owned);
                        let priority = Priority::new(row.priority)?;
                        let format = BookFormat::try_from(row.format.as_str())?;
                        let store = BookStore::try_from(row.store.as_str())?;

                        Book::new(
                            book_id,
                            title,
                            isbn,
                            read,
                            owned,
                            priority,
                            format,
                            store,
                            row.created_at,
                            row.updated_at,
                        )
                    },
                )
                .try_collect()
                .await;

        Ok(books?)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    #[ignore] // Depends on PostgreSQL
    async fn test_find_all() -> anyhow::Result<()> {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await?;
        let mut tx = pool.begin().await?;

        let user_id = UserId::new(String::from("user1"))?;
        let actual = InternalBookRepository::find_all(&user_id, &mut tx).await?;

        tx.rollback().await?;

        assert_eq!(actual.len(), 0);

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
