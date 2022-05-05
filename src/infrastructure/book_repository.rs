use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{PgConnection, PgPool};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        author::AuthorId,
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
    author_ids: Option<Vec<Uuid>>,
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
    async fn create(
        user_id: &UserId,
        book: &Book,
        conn: &mut PgConnection,
    ) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO book (
               id,
               user_id,
               title,
               isbn,
               read,
               owned,
               priority,
               format,
               store,
               created_at,
               updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);",
        )
        .bind(book.id().to_uuid())
        .bind(user_id.as_str())
        .bind(book.title().as_str())
        .bind(book.isbn().as_str())
        .bind(book.read().to_bool())
        .bind(book.owned().to_bool())
        .bind(book.priority().to_i32())
        .bind(book.format().to_string())
        .bind(book.store().to_string())
        .bind(book.created_at())
        .bind(book.updated_at())
        .execute(&mut *conn)
        .await?;

        let author_ids: Vec<Uuid> = book
            .author_ids()
            .iter()
            .map(|author_id| author_id.to_uuid())
            .collect();

        // TODO: Delete from here. Move to update function.
        // https://github.com/launchbadge/sqlx/blob/fa5c436918664de112677519d73cf6939c938cb0/FAQ.md#how-can-i-do-a-select--where-foo-in--query
        sqlx::query("DELETE FROM book_author WHERE book_id = $1 AND author_id != ALL($2)")
            .bind(book.id().to_uuid())
            .bind(&author_ids)
            .execute(&mut *conn)
            .await?;
        // https://github.com/launchbadge/sqlx/blob/fa5c436918664de112677519d73cf6939c938cb0/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts
        sqlx::query(
            "INSERT INTO book_author (book_id, author_id)
                    SELECT $1::uuid, * FROM UNNEST($2::uuid[])",
        )
        .bind(book.id().to_uuid())
        .bind(&author_ids)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    async fn find_all(user_id: &UserId, conn: &mut PgConnection) -> Result<Vec<Book>, DomainError> {
        let books: Result<Vec<Book>, DomainError> = sqlx::query_as(
            "SELECT * FROM book
                           LEFT OUTER JOIN
                           (SELECT book_id, array_agg(author_id) AS author_ids FROM book_author
                            GROUP BY book_author.book_id)
                           AS t1 ON book.id = t1.book_id
                           WHERE book.user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch(conn)
        .map(
            |row: Result<BookRow, sqlx::Error>| -> Result<Book, DomainError> {
                let row = row?;
                let book_id = BookId::new(row.id)?;
                let title = BookTitle::new(row.title)?;
                let author_ids: Vec<AuthorId> = row
                    .author_ids
                    .map(|author_ids| {
                        author_ids
                            .into_iter()
                            .map(|uuid| AuthorId::new(uuid))
                            .collect()
                    })
                    .unwrap_or_else(|| vec![]);
                let isbn = Isbn::new(row.isbn)?;
                let read = ReadFlag::new(row.read);
                let owned = OwnedFlag::new(row.owned);
                let priority = Priority::new(row.priority)?;
                let format = BookFormat::try_from(row.format.as_str())?;
                let store = BookStore::try_from(row.store.as_str())?;

                Book::new(
                    book_id,
                    title,
                    author_ids,
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

    use crate::{
        domain::entity::{
            author::{Author, AuthorName},
            user::User,
        },
        infrastructure::{
            author_repository::InternalAuthorRepository, user_repository::InternalUserRepository,
        },
    };

    use super::*;
    use sqlx::{postgres::PgPoolOptions, Postgres, Transaction};
    use time::{date, time};

    #[tokio::test]
    #[ignore] // Depends on PostgreSQL
    async fn test_create_and_find_all() -> anyhow::Result<()> {
        let mut tx = prepare_tx().await?;
        let user_id = prepare_user(&mut tx).await?;
        let author_ids1 = prepare_authors1(&user_id, &mut tx).await?;
        let author_ids2 = prepare_authors2(&user_id, &mut tx).await?;

        let all_books = InternalBookRepository::find_all(&user_id, &mut tx).await?;
        assert_eq!(all_books.len(), 0);

        let book1 = book_entity1(&author_ids1)?;
        let book2 = book_entity2(&author_ids2)?;
        InternalBookRepository::create(&user_id, &book1, &mut tx).await?;
        InternalBookRepository::create(&user_id, &book2, &mut tx).await?;

        let all_books = InternalBookRepository::find_all(&user_id, &mut tx).await?;
        assert_eq!(all_books.len(), 2);
        if all_books[0] == book1 {
            assert_eq!(all_books[0], book1);
            assert_eq!(all_books[1], book2);
        } else {
            assert_eq!(all_books[0], book2);
            assert_eq!(all_books[1], book1);
        }

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

    async fn prepare_authors1(
        user_id: &UserId,
        tx: &mut PgConnection,
    ) -> Result<Vec<AuthorId>, DomainError> {
        let author_id1 = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        let author_id2 = AuthorId::try_from("925aaf96-64c7-44be-85f8-767a20b2c20c")?;
        let author_ids = vec![author_id1.clone(), author_id2.clone()];
        let author1 = Author::new(author_id1, AuthorName::new("author1".to_owned())?)?;
        let author2 = Author::new(author_id2, AuthorName::new("author2".to_owned())?)?;
        InternalAuthorRepository::create(user_id, &author1, tx).await?;
        InternalAuthorRepository::create(user_id, &author2, tx).await?;

        Ok(author_ids)
    }

    async fn prepare_authors2(
        user_id: &UserId,
        tx: &mut PgConnection,
    ) -> Result<Vec<AuthorId>, DomainError> {
        let author_id1 = AuthorId::try_from("93090e87-b7a1-403c-974c-d74d881e83b9")?;
        let author_ids = vec![author_id1.clone()];
        let author1 = Author::new(author_id1, AuthorName::new("author1".to_owned())?)?;
        InternalAuthorRepository::create(user_id, &author1, tx).await?;

        Ok(author_ids)
    }

    fn book_entity1(author_ids: &Vec<AuthorId>) -> Result<Book, DomainError> {
        let book_id = BookId::try_from("675bc8d9-3155-42fb-87b0-0a82cb162848")?;
        let title = BookTitle::new("title1".to_owned())?;
        let isbn = Isbn::new("isbn".to_owned())?;
        let read = ReadFlag::new(false);
        let owned = OwnedFlag::new(false);
        let priority = Priority::new(50)?;
        let format = BookFormat::EBook;
        let store = BookStore::Kindle;
        let created_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00));
        let updated_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00));

        let book = Book::new(
            book_id,
            title,
            author_ids.clone(),
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            created_at,
            updated_at,
        )?;

        Ok(book)
    }

    fn book_entity2(author_ids: &Vec<AuthorId>) -> Result<Book, DomainError> {
        let book_id = BookId::try_from("c5a81e57-bc91-40ff-8b57-18cfa7cc7ae8")?;
        let title = BookTitle::new("title2".to_owned())?;
        let isbn = Isbn::new("isbn".to_owned())?;
        let read = ReadFlag::new(false);
        let owned = OwnedFlag::new(false);
        let priority = Priority::new(50)?;
        let format = BookFormat::EBook;
        let store = BookStore::Kindle;
        let created_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00));
        let updated_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00));

        let book = Book::new(
            book_id,
            title,
            author_ids.clone(),
            isbn,
            read,
            owned,
            priority,
            format,
            store,
            created_at,
            updated_at,
        )?;

        Ok(book)
    }
}
