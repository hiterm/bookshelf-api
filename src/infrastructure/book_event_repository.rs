use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::AuthorId,
            book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            event::{BookEvent, EventOperation},
            event_set::EventSetId,
            user::UserId,
        },
        error::DomainError,
        repository::book_event_repository::BookEventRepository,
    },
};

#[derive(sqlx::FromRow)]
struct BookEventRow {
    event_id: i64,
    event_set_id: Uuid,
    operation: String,
    book_id: Uuid,
    title: Option<String>,
    isbn: Option<String>,
    read: Option<bool>,
    owned: Option<bool>,
    priority: Option<i32>,
    format: Option<String>,
    store: Option<String>,
    book_created_at: Option<OffsetDateTime>,
    book_updated_at: Option<OffsetDateTime>,
    changed_at: OffsetDateTime,
    author_ids: Option<Vec<Uuid>>,
    extra: Option<Value>,
}

fn row_to_book_event(row: BookEventRow) -> Result<BookEvent, DomainError> {
    let operation =
        EventOperation::try_from(row.operation.as_str()).map_err(DomainError::Unexpected)?;
    let book_id = BookId::new(row.book_id)?;

    let title = row.title.map(BookTitle::new).transpose()?;
    let isbn = row.isbn.map(Isbn::new).transpose()?;
    let priority = row.priority.map(Priority::new).transpose()?;
    let format = row
        .format
        .map(|s| BookFormat::try_from(s.as_str()))
        .transpose()?;
    let store = row
        .store
        .map(|s| BookStore::try_from(s.as_str()))
        .transpose()?;
    let author_ids: Vec<AuthorId> = row
        .author_ids
        .unwrap_or_default()
        .into_iter()
        .map(AuthorId::new)
        .collect();

    Ok(BookEvent {
        event_id: row.event_id,
        event_set_id: EventSetId::from(row.event_set_id),
        operation,
        book_id,
        title,
        author_ids,
        isbn,
        read: row.read.map(ReadFlag::new),
        owned: row.owned.map(OwnedFlag::new),
        priority,
        format,
        store,
        book_created_at: row.book_created_at,
        book_updated_at: row.book_updated_at,
        changed_at: row.changed_at,
        extra: row.extra,
    })
}

#[derive(Debug, Clone)]
pub struct PgBookEventRepository {
    pool: PgPool,
}

impl PgBookEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BookEventRepository for PgBookEventRepository {
    async fn find_by_book(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Vec<BookEvent>, DomainError> {
        let rows: Vec<BookEventRow> = sqlx::query_as(
            "SELECT
                be.event_id,
                be.event_set_id,
                be.operation,
                be.book_id,
                be.title,
                be.isbn,
                be.read,
                be.owned,
                be.priority,
                be.format,
                be.store,
                be.book_created_at,
                be.book_updated_at,
                be.changed_at,
                array_agg(bea.author_id) FILTER (WHERE bea.author_id IS NOT NULL) AS author_ids,
                be.extra
            FROM book_event be
            LEFT JOIN book_event_author bea ON be.event_id = bea.event_id
            WHERE be.user_id = $1 AND be.book_id = $2
            GROUP BY be.event_id
            ORDER BY be.changed_at DESC",
        )
        .bind(user_id.as_str())
        .bind(book_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_book_event).collect()
    }

    async fn find_by_event_id(
        &self,
        user_id: &UserId,
        event_id: i64,
    ) -> Result<Option<BookEvent>, DomainError> {
        let row: Option<BookEventRow> = sqlx::query_as(
            "SELECT
                be.event_id,
                be.event_set_id,
                be.operation,
                be.book_id,
                be.title,
                be.isbn,
                be.read,
                be.owned,
                be.priority,
                be.format,
                be.store,
                be.book_created_at,
                be.book_updated_at,
                be.changed_at,
                array_agg(bea.author_id) FILTER (WHERE bea.author_id IS NOT NULL) AS author_ids,
                be.extra
            FROM book_event be
            LEFT JOIN book_event_author bea ON be.event_id = bea.event_id
            WHERE be.user_id = $1 AND be.event_id = $2
            GROUP BY be.event_id",
        )
        .bind(user_id.as_str())
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_book_event).transpose()
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
                event::{EventOperation, EventSetOperation},
                user::User,
            },
            error::DomainError,
            repository::{
                author_repository::AuthorRepository, book_event_repository::BookEventRepository,
                book_repository::BookRepository, transaction::TransactionManager,
                user_repository::UserRepository,
            },
        },
        infrastructure::{
            author_repository::PgAuthorRepository, book_repository::PgBookRepository,
            transaction::PgTransactionManager, user_repository::PgUserRepository,
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

    // Wrap a BookRepository mutation in a single transaction opened via
    // PgTransactionManager, mirroring how the use-case layer drives it.
    async fn create_book(
        pool: &PgPool,
        book_repo: &PgBookRepository,
        user_id: &UserId,
        book: &Book,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::CreateBook).await?;
        book_repo.create(&mut tx, user_id, book).await?;
        tm.commit(tx).await
    }

    async fn update_book(
        pool: &PgPool,
        book_repo: &PgBookRepository,
        user_id: &UserId,
        book: &Book,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::UpdateBook).await?;
        book_repo.update(&mut tx, user_id, book).await?;
        tm.commit(tx).await
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
    async fn find_by_book_returns_events_ordered_desc(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let event_repo = PgBookEventRepository::new(pool.clone());

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
        create_book(&pool, &book_repo, &user_id, &book).await?;

        let updated = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            "updated",
            &[author_id],
        )?;
        update_book(&pool, &book_repo, &user_id, &updated).await?;

        let entries = event_repo.find_by_book(&user_id, book.id()).await?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].operation, EventOperation::Update);
        // post-update state
        assert_eq!(entries[0].title.as_ref().unwrap().as_str(), "updated");
        assert_eq!(entries[1].operation, EventOperation::Create);
        assert_eq!(entries[1].title.as_ref().unwrap().as_str(), "original");

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_book_aggregates_multiple_author_ids(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let event_repo = PgBookEventRepository::new(pool.clone());

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
        create_book(&pool, &book_repo, &user_id, &book).await?;

        let entries = event_repo.find_by_book(&user_id, book.id()).await?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].author_ids.len(), 2);
        assert!(entries[0].author_ids.contains(&author_id1));
        assert!(entries[0].author_ids.contains(&author_id2));

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_book_returns_empty_for_unknown_book(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let event_repo = PgBookEventRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let unknown_book_id = BookId::try_from("00000000-0000-0000-0000-000000000000")?;

        let entries = event_repo.find_by_book(&user_id, &unknown_book_id).await?;
        assert!(entries.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_event_id_returns_correct_entry(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let event_repo = PgBookEventRepository::new(pool.clone());

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
        create_book(&pool, &book_repo, &user_id, &book).await?;

        let (event_id,): (i64,) =
            sqlx::query_as("SELECT event_id FROM book_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;

        let entry = event_repo.find_by_event_id(&user_id, event_id).await?;
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.event_id, event_id);
        assert_eq!(entry.title.as_ref().unwrap().as_str(), "title1");
        assert_eq!(entry.operation, EventOperation::Create);

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_event_id_returns_none_for_wrong_user(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let event_repo = PgBookEventRepository::new(pool.clone());

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
        create_book(&pool, &book_repo, &user1_id, &book).await?;

        let (event_id,): (i64,) =
            sqlx::query_as("SELECT event_id FROM book_event WHERE user_id = $1")
                .bind(user1_id.as_str())
                .fetch_one(&pool)
                .await?;

        // user2 must not see user1's event entry
        let entry = event_repo.find_by_event_id(&user2_id, event_id).await?;
        assert!(entry.is_none());

        Ok(())
    }
}
