use std::ops::DerefMut;

use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{PgConnection, PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::AuthorId,
            book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            user::UserId,
        },
        error::DomainError,
        repository::book_repository::BookRepository,
    },
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
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct PgBookRepository {
    pool: PgPool,
}

impl PgBookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookRepository for PgBookRepository {
    async fn create(&self, user_id: &UserId, book: &Book) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await?;
        InternalBookRepository::create(user_id, book, &mut tx).await?;
        tx.commit().await?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Option<Book>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        InternalBookRepository::find_by_id(user_id, book_id, &mut conn).await
    }

    async fn find_all(&self, user_id: &UserId) -> Result<Vec<Book>, DomainError> {
        let mut conn = self.pool.acquire().await?;
        InternalBookRepository::find_all(user_id, &mut conn).await
    }

    async fn update(&self, user_id: &UserId, book: &Book) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await?;
        InternalBookRepository::update(user_id, book, &mut tx).await?;
        tx.commit().await?;

        Ok(())
    }

    async fn delete(&self, user_id: &UserId, book_id: &BookId) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await?;
        InternalBookRepository::delete(user_id, book_id, &mut tx).await?;
        tx.commit().await?;

        Ok(())
    }
}

pub struct InternalBookRepository {}

impl InternalBookRepository {
    pub async fn create(
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

        // https://github.com/launchbadge/sqlx/blob/fa5c436918664de112677519d73cf6939c938cb0/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts
        sqlx::query(
            "INSERT INTO book_author (user_id, book_id, author_id)
                    SELECT $1, $2::uuid, * FROM UNNEST($3::uuid[])",
        )
        .bind(user_id.as_str())
        .bind(book.id().to_uuid())
        .bind(&author_ids)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    async fn find_by_id(
        user_id: &UserId,
        book_id: &BookId,
        conn: &mut PgConnection,
    ) -> Result<Option<Book>, DomainError> {
        let book_row: Option<BookRow> = sqlx::query_as(
            "WITH book_of_user AS(
                SELECT
                    *
                FROM
                    book
                WHERE
                    book.user_id = $1
            ),
            authors_of_book_and_user AS(
                SELECT
                    book_id,
                    array_agg(author_id) AS author_ids
                FROM
                    book_author
                WHERE
                    book_author.user_id = $1
                GROUP BY
                    book_author.book_id
            )
            SELECT
                *
            FROM
                book_of_user
                LEFT OUTER JOIN
                    authors_of_book_and_user
                ON  book_of_user.id = authors_of_book_and_user.book_id
            WHERE book_of_user.id = $2",
        )
        .bind(user_id.as_str())
        .bind(book_id.to_uuid())
        .fetch_optional(conn)
        .await?;

        let book = book_row.map(|row| {
            let book_id = BookId::new(row.id)?;
            let title = BookTitle::new(row.title)?;
            let author_ids: Vec<AuthorId> = row
                .author_ids
                .map(|author_ids| author_ids.into_iter().map(AuthorId::new).collect())
                .unwrap_or_default();
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
        });
        let book = book.transpose()?;

        Ok(book)
    }

    async fn find_all(user_id: &UserId, conn: &mut PgConnection) -> Result<Vec<Book>, DomainError> {
        let books: Result<Vec<Book>, DomainError> = sqlx::query_as(
            "WITH book_of_user AS(
                SELECT
                    *
                FROM
                    book
                WHERE
                    book.user_id = $1
            ),
            authors_of_book_and_user AS(
                SELECT
                    book_id,
                    array_agg(author_id) AS author_ids
                FROM
                    book_author
                WHERE
                    book_author.user_id = $1
                GROUP BY
                    book_author.book_id
            )
            SELECT
                *
            FROM
                book_of_user
                LEFT OUTER JOIN
                    authors_of_book_and_user
                ON  book_of_user.id = authors_of_book_and_user.book_id",
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
                    .map(|author_ids| author_ids.into_iter().map(AuthorId::new).collect())
                    .unwrap_or_else(std::vec::Vec::new);
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

        books
    }

    async fn update(
        user_id: &UserId,
        book: &Book,
        conn: &mut PgConnection,
    ) -> Result<(), DomainError> {
        let result = sqlx::query(
            "UPDATE book SET
               user_id = $1,
               title = $2,
               isbn = $3,
               read = $4,
               owned = $5,
               priority = $6,
               format = $7,
               store = $8,
               created_at = $9,
               updated_at = $10
            WHERE id = $11",
        )
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
        .bind(book.id().to_uuid())
        .execute(&mut *conn)
        .await?;

        let rows_affected = result.rows_affected();
        match rows_affected {
            0 => {
                return Err(DomainError::NotFound {
                    entity_type: "book",
                    entity_id: book.id().to_string(),
                    user_id: user_id.to_owned().into_string(),
                });
            }
            1 => {}
            _ => {
                return Err(DomainError::Unexpected(String::from(
                    "rows_affected is greater than 1.",
                )))
            }
        }

        let author_ids: Vec<Uuid> = book
            .author_ids()
            .iter()
            .map(|author_id| author_id.to_uuid())
            .collect();

        // https://github.com/launchbadge/sqlx/blob/fa5c436918664de112677519d73cf6939c938cb0/FAQ.md#how-can-i-do-a-select--where-foo-in--query
        sqlx::query("DELETE FROM book_author WHERE book_id = $1 AND author_id != ALL($2)")
            .bind(book.id().to_uuid())
            .bind(&author_ids)
            .execute(&mut *conn)
            .await?;

        // https://github.com/launchbadge/sqlx/blob/fa5c436918664de112677519d73cf6939c938cb0/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts
        sqlx::query(
            "INSERT INTO book_author (user_id, book_id, author_id)
                    SELECT $1, $2::uuid, * FROM UNNEST($3::uuid[])
            ON CONFLICT DO NOTHING",
        )
        .bind(user_id.as_str())
        .bind(book.id().to_uuid())
        .bind(&author_ids)
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    async fn delete(
        user_id: &UserId,
        book_id: &BookId,
        conn: &mut Transaction<'_, Postgres>,
    ) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM book_author WHERE user_id = $1 AND book_id = $2")
            .bind(user_id.as_str())
            .bind(book_id.to_uuid())
            .execute(conn.deref_mut())
            .await?;

        let result = sqlx::query("DELETE FROM book WHERE user_id = $1 AND id = $2")
            .bind(user_id.as_str())
            .bind(book_id.to_uuid())
            .execute(conn.deref_mut())
            .await?;

        let rows_affected = result.rows_affected();
        match rows_affected {
            0 => {
                return Err(DomainError::NotFound {
                    entity_type: "book",
                    entity_id: book_id.to_string(),
                    user_id: user_id.to_owned().into_string(),
                });
            }
            1 => {}
            _ => {
                return Err(DomainError::Unexpected(String::from(
                    "rows_affected is greater than 1.",
                )))
            }
        }

        Ok(())
    }
}

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {

    use crate::{
        domain::{
            entity::{
                author::{Author, AuthorName},
                user::User,
            },
            repository::{author_repository::AuthorRepository, user_repository::UserRepository},
        },
        infrastructure::{
            author_repository::PgAuthorRepository, user_repository::PgUserRepository,
        },
    };

    use super::*;
    use time::{
        macros::{date, time},
        PrimitiveDateTime,
    };

    #[sqlx::test]
    async fn test_create_and_find_by_id(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository).await?;
        let author_ids = prepare_authors1(&user_id, &author_repository).await?;

        let all_books = book_repository.find_all(&user_id).await?;
        assert_eq!(all_books.len(), 0);

        let book = book_entity1(&author_ids)?;
        book_repository.create(&user_id, &book).await?;

        let actual = book_repository.find_by_id(&user_id, book.id()).await?;
        assert_eq!(actual, Some(book));

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_and_find_all(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository).await?;

        let author_ids1 = prepare_authors1(&user_id, &author_repository).await?;
        let author_ids2 = prepare_authors2(&user_id, &author_repository).await?;

        let all_books = book_repository.find_all(&user_id).await?;
        assert_eq!(all_books.len(), 0);

        let book1 = book_entity1(&author_ids1)?;
        let book2 = book_entity2(&author_ids2)?;
        book_repository.create(&user_id, &book1).await?;
        book_repository.create(&user_id, &book2).await?;

        let all_books = book_repository.find_all(&user_id).await?;
        assert_eq!(all_books.len(), 2);
        if all_books[0] == book1 {
            assert_eq!(all_books[0], book1);
            assert_eq!(all_books[1], book2);
        } else {
            assert_eq!(all_books[0], book2);
            assert_eq!(all_books[1], book1);
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) -> anyhow::Result<()> {
        // setup
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository).await?;
        let mut author_ids = prepare_authors1(&user_id, &author_repository).await?;
        let mut book = book_entity1(&author_ids)?;
        book_repository.create(&user_id, &book).await?;
        let actual = book_repository.find_by_id(&user_id, book.id()).await?;
        assert_eq!(actual, Some(book.clone()));

        // update
        book.set_title(BookTitle::new("another_title".to_owned())?);
        author_ids.pop();
        let another_author_id = AuthorId::try_from("e30ce456-d34a-4c42-831c-b08d5f9ed81f")?;
        let another_author = Author::new(
            another_author_id.clone(),
            AuthorName::new("another_author1".to_owned())?,
        )?;
        author_repository.create(&user_id, &another_author).await?;
        author_ids.push(another_author_id);
        book.set_author_ids(author_ids);
        book_repository.update(&user_id, &book).await?;

        let actual = book_repository.find_by_id(&user_id, book.id()).await?;
        assert_eq!(actual, Some(book.clone()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) -> anyhow::Result<()> {
        let user_repository = PgUserRepository::new(pool.clone());
        let author_repository = PgAuthorRepository::new(pool.clone());
        let book_repository = PgBookRepository::new(pool.clone());

        let user_id = prepare_user(&user_repository).await?;
        let author_ids = prepare_authors1(&user_id, &author_repository).await?;
        let book = book_entity1(&author_ids)?;
        book_repository.create(&user_id, &book).await?;
        let actual = book_repository.find_by_id(&user_id, book.id()).await?;
        assert_eq!(actual, Some(book.clone()));

        book_repository.delete(&user_id, book.id()).await?;
        let actual = book_repository.find_by_id(&user_id, book.id()).await?;
        assert_eq!(actual, None);

        Ok(())
    }

    async fn prepare_user(repository: &PgUserRepository) -> Result<UserId, DomainError> {
        let user_id = UserId::new(String::from("user1"))?;
        let user = User::new(user_id.clone());
        repository.create(&user).await?;

        Ok(user_id)
    }

    async fn prepare_authors1(
        user_id: &UserId,
        repository: &PgAuthorRepository,
    ) -> Result<Vec<AuthorId>, DomainError> {
        let author_id1 = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        let author_id2 = AuthorId::try_from("925aaf96-64c7-44be-85f8-767a20b2c20c")?;
        let author_ids = vec![author_id1.clone(), author_id2.clone()];
        let author1 = Author::new(author_id1, AuthorName::new("author1".to_owned())?)?;
        let author2 = Author::new(author_id2, AuthorName::new("author2".to_owned())?)?;
        repository.create(user_id, &author1).await?;
        repository.create(user_id, &author2).await?;

        Ok(author_ids)
    }

    async fn prepare_authors2(
        user_id: &UserId,
        repository: &PgAuthorRepository,
    ) -> Result<Vec<AuthorId>, DomainError> {
        let author_id1 = AuthorId::try_from("93090e87-b7a1-403c-974c-d74d881e83b9")?;
        let author_ids = vec![author_id1.clone()];
        let author1 = Author::new(author_id1, AuthorName::new("author1".to_owned())?)?;
        repository.create(user_id, &author1).await?;

        Ok(author_ids)
    }

    fn book_entity1(author_ids: &[AuthorId]) -> Result<Book, DomainError> {
        let book_id = BookId::try_from("675bc8d9-3155-42fb-87b0-0a82cb162848")?;
        let title = BookTitle::new("title1".to_owned())?;
        let isbn = Isbn::new("1111111111116".to_owned())?;
        let read = ReadFlag::new(false);
        let owned = OwnedFlag::new(false);
        let priority = Priority::new(50)?;
        let format = BookFormat::EBook;
        let store = BookStore::Kindle;
        let created_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        let updated_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();

        let book = Book::new(
            book_id,
            title,
            author_ids.to_owned(),
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

    fn book_entity2(author_ids: &[AuthorId]) -> Result<Book, DomainError> {
        let book_id = BookId::try_from("c5a81e57-bc91-40ff-8b57-18cfa7cc7ae8")?;
        let title = BookTitle::new("title2".to_owned())?;
        let isbn = Isbn::new("2222222222222".to_owned())?;
        let read = ReadFlag::new(false);
        let owned = OwnedFlag::new(false);
        let priority = Priority::new(50)?;
        let format = BookFormat::EBook;
        let store = BookStore::Kindle;
        let created_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        let updated_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();

        let book = Book::new(
            book_id,
            title,
            author_ids.to_owned(),
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
