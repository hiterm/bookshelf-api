use futures_util::{StreamExt, TryStreamExt};
use sqlx::{PgConnection, PgPool};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        book::{Book, BookId, BookTitle},
        user::UserId,
    },
    error::DomainError,
};

#[derive(sqlx::FromRow)]
struct BookRow {
    id: Uuid,
    user_id: String,
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

struct BookRepository {
    pool: PgPool,
}

impl BookRepository {
    async fn find_all_book_rows(conn: &mut PgConnection) -> Vec<BookRow> {
        let book_rows: Vec<BookRow> = sqlx::query_as("SELECT * FROM book")
            .fetch_all(conn)
            .await
            .unwrap();
        book_rows
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
                        let book_title = BookTitle::new(row.title)?;
                        todo!()
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
    async fn test_find_all() {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();

        let actual = BookRepository::find_all_book_rows(&mut tx).await;

        tx.rollback().await.unwrap();

        assert_eq!(actual.len(), 0);
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
