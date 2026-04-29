use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::AuthorId,
            book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            change_set::ChangeSetId,
            history::{BookHistory, HistoryOperation},
            user::UserId,
        },
        error::DomainError,
        repository::book_history_repository::BookHistoryRepository,
    },
};

#[derive(sqlx::FromRow)]
struct BookHistoryRow {
    history_id: i64,
    change_set_id: Uuid,
    operation: String,
    book_id: Uuid,
    title: String,
    isbn: String,
    read: bool,
    owned: bool,
    priority: i32,
    format: String,
    store: String,
    book_created_at: OffsetDateTime,
    book_updated_at: OffsetDateTime,
    changed_at: OffsetDateTime,
    author_ids: Option<Vec<Uuid>>,
}

fn row_to_book_history(row: BookHistoryRow) -> Result<BookHistory, DomainError> {
    let operation =
        HistoryOperation::try_from(row.operation.as_str()).map_err(DomainError::Unexpected)?;
    let book_id = BookId::new(row.book_id)?;
    let title = BookTitle::new(row.title)?;
    let isbn = Isbn::new(row.isbn)?;
    let priority = Priority::new(row.priority)?;
    let format = BookFormat::try_from(row.format.as_str())?;
    let store = BookStore::try_from(row.store.as_str())?;
    let author_ids: Vec<AuthorId> = row
        .author_ids
        .unwrap_or_default()
        .into_iter()
        .map(AuthorId::new)
        .collect();

    Ok(BookHistory {
        history_id: row.history_id,
        change_set_id: ChangeSetId::from(row.change_set_id),
        operation,
        book_id,
        title,
        author_ids,
        isbn,
        read: ReadFlag::new(row.read),
        owned: OwnedFlag::new(row.owned),
        priority,
        format,
        store,
        book_created_at: row.book_created_at,
        book_updated_at: row.book_updated_at,
        changed_at: row.changed_at,
    })
}

#[derive(Debug, Clone)]
pub struct PgBookHistoryRepository {
    pool: PgPool,
}

impl PgBookHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookHistoryRepository for PgBookHistoryRepository {
    async fn find_by_book(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Vec<BookHistory>, DomainError> {
        let rows: Vec<BookHistoryRow> = sqlx::query_as(
            "SELECT
                bh.history_id,
                bh.change_set_id,
                bh.operation,
                bh.book_id,
                bh.title,
                bh.isbn,
                bh.read,
                bh.owned,
                bh.priority,
                bh.format,
                bh.store,
                bh.book_created_at,
                bh.book_updated_at,
                bh.changed_at,
                array_agg(bha.author_id) FILTER (WHERE bha.author_id IS NOT NULL) AS author_ids
            FROM book_history bh
            LEFT JOIN book_history_author bha ON bh.history_id = bha.history_id
            WHERE bh.user_id = $1 AND bh.book_id = $2
            GROUP BY bh.history_id
            ORDER BY bh.changed_at DESC",
        )
        .bind(user_id.as_str())
        .bind(book_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_book_history).collect()
    }

    async fn find_by_history_id(
        &self,
        user_id: &UserId,
        history_id: i64,
    ) -> Result<Option<BookHistory>, DomainError> {
        let row: Option<BookHistoryRow> = sqlx::query_as(
            "SELECT
                bh.history_id,
                bh.change_set_id,
                bh.operation,
                bh.book_id,
                bh.title,
                bh.isbn,
                bh.read,
                bh.owned,
                bh.priority,
                bh.format,
                bh.store,
                bh.book_created_at,
                bh.book_updated_at,
                bh.changed_at,
                array_agg(bha.author_id) FILTER (WHERE bha.author_id IS NOT NULL) AS author_ids
            FROM book_history bh
            LEFT JOIN book_history_author bha ON bh.history_id = bha.history_id
            WHERE bh.user_id = $1 AND bh.history_id = $2
            GROUP BY bh.history_id",
        )
        .bind(user_id.as_str())
        .bind(history_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_book_history).transpose()
    }
}

#[cfg(feature = "test-with-database")]
#[cfg(test)]
mod tests {
    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            entity::{
                author::{Author, AuthorId, AuthorName},
                book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                history::HistoryOperation,
                user::User,
            },
            error::DomainError,
            repository::{
                author_repository::AuthorRepository,
                book_history_repository::BookHistoryRepository, book_repository::BookRepository,
                user_repository::UserRepository,
            },
        },
        infrastructure::{
            author_repository::PgAuthorRepository, book_repository::PgBookRepository,
            user_repository::PgUserRepository,
        },
    };
    use time::{
        PrimitiveDateTime,
        macros::{date, time},
    };

    use super::*;

    async fn prepare_user(repository: &PgUserRepository, id: &str) -> Result<UserId, DomainError> {
        let user_id = UserId::new(id.to_string())?;
        let user = User::new(user_id.clone());
        repository.create(&user).await?;
        Ok(user_id)
    }

    fn make_book(
        book_id_str: &str,
        title: &str,
        author_ids: &[AuthorId],
    ) -> Result<Book, DomainError> {
        let created_at = PrimitiveDateTime::new(date!(2022 - 05 - 05), time!(0:00)).assume_utc();
        Book::new(
            BookId::try_from(book_id_str)?,
            BookTitle::new(title.to_owned())?,
            author_ids.to_vec(),
            Isbn::new("1111111111116".to_owned())?,
            ReadFlag::new(false),
            OwnedFlag::new(false),
            Priority::new(50)?,
            BookFormat::EBook,
            BookStore::Kindle,
            created_at,
            created_at,
        )
    }

    #[sqlx::test]
    async fn find_by_book_returns_history_ordered_desc(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let history_repo = PgBookHistoryRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        author_repo
            .create(
                &user_id,
                &Author::new(author_id.clone(), AuthorName::new("author1".to_owned())?)?,
            )
            .await?;

        let book = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            "original",
            &[author_id.clone()],
        )?;
        book_repo.create(&user_id, &book).await?;

        let updated = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            "updated",
            &[author_id],
        )?;
        book_repo.update(&user_id, &updated).await?;

        let entries = history_repo.find_by_book(&user_id, book.id()).await?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].operation, HistoryOperation::Update);
        assert_eq!(entries[0].title.as_str(), "original"); // pre-update snapshot
        assert_eq!(entries[1].operation, HistoryOperation::Create);
        assert_eq!(entries[1].title.as_str(), "original");

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_book_aggregates_multiple_author_ids(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let history_repo = PgBookHistoryRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let author_id1 = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        let author_id2 = AuthorId::try_from("925aaf96-64c7-44be-85f8-767a20b2c20c")?;
        author_repo
            .create(
                &user_id,
                &Author::new(author_id1.clone(), AuthorName::new("a1".to_owned())?)?,
            )
            .await?;
        author_repo
            .create(
                &user_id,
                &Author::new(author_id2.clone(), AuthorName::new("a2".to_owned())?)?,
            )
            .await?;

        let book = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            "title1",
            &[author_id1.clone(), author_id2.clone()],
        )?;
        book_repo.create(&user_id, &book).await?;

        let entries = history_repo.find_by_book(&user_id, book.id()).await?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].author_ids.len(), 2);
        assert!(entries[0].author_ids.contains(&author_id1));
        assert!(entries[0].author_ids.contains(&author_id2));

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_book_returns_empty_for_unknown_book(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let history_repo = PgBookHistoryRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let unknown_book_id = BookId::try_from("00000000-0000-0000-0000-000000000000")?;

        let entries = history_repo
            .find_by_book(&user_id, &unknown_book_id)
            .await?;
        assert!(entries.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_history_id_returns_correct_entry(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let history_repo = PgBookHistoryRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        author_repo
            .create(
                &user_id,
                &Author::new(author_id.clone(), AuthorName::new("author1".to_owned())?)?,
            )
            .await?;

        let book = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            "title1",
            &[author_id],
        )?;
        book_repo.create(&user_id, &book).await?;

        let (history_id,): (i64,) =
            sqlx::query_as("SELECT history_id FROM book_history WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;

        let entry = history_repo
            .find_by_history_id(&user_id, history_id)
            .await?;
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.history_id, history_id);
        assert_eq!(entry.title.as_str(), "title1");
        assert_eq!(entry.operation, HistoryOperation::Create);

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_history_id_returns_none_for_wrong_user(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let history_repo = PgBookHistoryRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repo, "user1").await?;
        let user2_id = prepare_user(&user_repo, "user2").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        author_repo
            .create(
                &user1_id,
                &Author::new(author_id.clone(), AuthorName::new("author1".to_owned())?)?,
            )
            .await?;

        let book = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            "title1",
            &[author_id],
        )?;
        book_repo.create(&user1_id, &book).await?;

        let (history_id,): (i64,) =
            sqlx::query_as("SELECT history_id FROM book_history WHERE user_id = $1")
                .bind(user1_id.as_str())
                .fetch_one(&pool)
                .await?;

        // user2 must not see user1's history entry
        let entry = history_repo
            .find_by_history_id(&user2_id, history_id)
            .await?;
        assert!(entry.is_none());

        Ok(())
    }
}
