use sqlx::Executor;
use time::PrimitiveDateTime;

use crate::domain::book::Book;

struct BookRepository<E> {
    executor: E,
}

impl<'a, E> BookRepository<E>
where
    E: Executor<'a>,
{
    fn findAll(&self) -> Vec<Book> {
        let book_rows: Vec<BookRow> = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_all(self.executor)
            .await
            .unwrap();

        todo!()
    }
}

#[derive(sqlx::FromRow)]
struct BookRow {
    id: String,
    user_id: String,
    isbn: Option<String>,
    read: bool,
    owned: bool,
    priority: u32,
    format: Option<String>,
    store: Option<String>,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

#[cfg(test)]
mod tests {
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn connect() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(row.0, 150);
    }
}
