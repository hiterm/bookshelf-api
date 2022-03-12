use sqlx::{
    database::HasArguments, Acquire, ColumnIndex, Database, Executor, IntoArguments, PgConnection,
    PgExecutor, Pool, Postgres, Type,
};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::domain::book::Book;

struct BookRepository<E> {
    executor: E,
}

impl<'a, E> BookRepository<E>
where
    E: Executor<'a>,
    <E::Database as HasArguments<'a>>::Arguments: IntoArguments<'a, <E as Executor<'a>>::Database>,
{
    // async fn findAll(&self) -> Vec<Book> {
    //     let book_rows: Vec<BookRow> = sqlx::query_as("SELECT 1")
    //         .fetch_all(self.executor)
    //         .await
    //         .unwrap();

    //     todo!()
    // }

    // async fn find_by_id(&self) -> Book {
    //     self.executor.fetch_one("SELECT 1").await;
    //     // let book_rows = sqlx::query("SELECT 1")
    //     //     .fetch_one(self.executor)
    //     //     .await
    //     //     .unwrap();

    //     todo!()
    // }
}

// async fn find_by_id<'a, E>(executor: E) -> Book
// where
//     E: Executor<'a>,
//     <E::Database as HasArguments<'a>>::Arguments: IntoArguments<'a, <E as Executor<'a>>::Database>,
// {
//     let book_rows = sqlx::query("SELECT 1").fetch_one(executor).await.unwrap();
//     todo!()
// }

// async fn find_by_id<DB>(pool: &Pool<DB>)
// where
//     DB: Database,
//     <DB as HasArguments<'_>>::Arguments: IntoArguments<'_, DB>,
//     &'c mut <DB as sqlx::Database>::Connection: for<'c> Executor<'c>,
// {
//     sqlx::query("SELECT 1").fetch_one(pool).await.unwrap();
// }

// async fn find_by_id<'a, A>(conn: A) -> Book
// where
//     A: Acquire<'a, Database = Postgres> + Send + 'a,
// {
//     let mut conn = conn.acquire().await.unwrap();

//     let book_row: (i32,) = sqlx::query_as("SELECT 1")
//         .fetch_one(&mut *conn)
//         .await
//         .unwrap();
//     todo!()
// }

async fn query_1(conn: &mut PgConnection) -> i32 {
    let book_row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(conn).await.unwrap();
    book_row.0
}

// #[derive(sqlx::FromRow)]
struct BookRow {
    id: Uuid,
    user_id: String,
    isbn: Option<String>,
    read: bool,
    owned: bool,
    priority: i32,
    format: Option<String>,
    store: Option<String>,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn connect() {
        dotenv::dotenv().ok();

        let db_url = fetch_database_url();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_timeout(Duration::from_secs(1))
            .connect(&db_url)
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();

        let actual = query_1(&mut tx).await;

        tx.rollback().await.unwrap();

        assert_eq!(actual, 1);
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
