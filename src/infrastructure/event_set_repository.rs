use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{
    entity::{
        event::EventSetOperation,
        event_set::{EventSet, EventSetId},
        user::UserId,
    },
    error::DomainError,
    repository::event_set_repository::EventSetRepository,
};

#[derive(sqlx::FromRow)]
struct EventSetRow {
    id: Uuid,
    user_id: String,
    operation: String,
    created_at: OffsetDateTime,
}

fn row_to_event_set(row: EventSetRow) -> Result<EventSet, DomainError> {
    Ok(EventSet {
        id: EventSetId::from(row.id),
        user_id: UserId::new(row.user_id)?,
        operation: EventSetOperation::try_from(row.operation.as_str())
            .map_err(DomainError::Unexpected)?,
        created_at: row.created_at,
    })
}

#[derive(Debug, Clone)]
pub struct PgEventSetRepository {
    pool: PgPool,
}

impl PgEventSetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventSetRepository for PgEventSetRepository {
    async fn find_all(&self, user_id: &UserId) -> Result<Vec<EventSet>, DomainError> {
        let rows: Vec<EventSetRow> = sqlx::query_as(
            "SELECT id, user_id, operation, created_at
             FROM event_set
             WHERE user_id = $1
             ORDER BY created_at DESC",
        )
        .bind(user_id.as_str())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_event_set).collect()
    }

    async fn find_by_id(
        &self,
        user_id: &UserId,
        event_set_id: &EventSetId,
    ) -> Result<Option<EventSet>, DomainError> {
        let row: Option<EventSetRow> = sqlx::query_as(
            "SELECT id, user_id, operation, created_at
             FROM event_set
             WHERE user_id = $1 AND id = $2",
        )
        .bind(user_id.as_str())
        .bind(event_set_id.to_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_event_set).transpose()
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
                event::EventSetOperation,
                user::User,
            },
            error::DomainError,
            repository::{
                author_repository::AuthorRepository, book_repository::BookRepository,
                transaction::TransactionManager, user_repository::UserRepository,
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

    async fn create_author(
        pool: &PgPool,
        author_repo: &PgAuthorRepository,
        user_id: &UserId,
        author: &Author,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::CreateAuthor).await?;
        author_repo.create(&mut tx, author).await?;
        tm.commit(tx).await
    }

    async fn create_book(
        pool: &PgPool,
        book_repo: &PgBookRepository,
        user_id: &UserId,
        book: &Book,
    ) -> Result<(), DomainError> {
        let tm = PgTransactionManager::new(pool.clone());
        let mut tx = tm.begin(user_id, EventSetOperation::CreateBook).await?;
        book_repo.create(&mut tx, book).await?;
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
    async fn find_all_returns_event_sets_ordered_desc(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let book_repo = PgBookRepository::new(pool.clone());
        let event_set_repo = PgEventSetRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        // First event set: create author.
        create_author(
            &pool,
            &author_repo,
            &user_id,
            &Author::new(author_id.clone(), AuthorName::new("author1".to_owned())?)?,
        )
        .await?;
        // Second event set: create book.
        let book = make_book(
            "675bc8d9-3155-42fb-87b0-0a82cb162848",
            "title1",
            &[author_id],
        )?;
        create_book(&pool, &book_repo, &user_id, &book).await?;

        let event_sets = event_set_repo.find_all(&user_id).await?;
        assert_eq!(event_sets.len(), 2);
        // Newest first: the book creation event set precedes the author one.
        assert_eq!(event_sets[0].operation, EventSetOperation::CreateBook);
        assert_eq!(event_sets[1].operation, EventSetOperation::CreateAuthor);
        assert!(event_sets[0].created_at >= event_sets[1].created_at);

        Ok(())
    }

    #[sqlx::test]
    async fn find_all_returns_empty_for_user_without_events(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let event_set_repo = PgEventSetRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;

        let event_sets = event_set_repo.find_all(&user_id).await?;
        assert!(event_sets.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_id_returns_event_set_for_owner(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let event_set_repo = PgEventSetRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        create_author(
            &pool,
            &author_repo,
            &user_id,
            &Author::new(author_id, AuthorName::new("author1".to_owned())?)?,
        )
        .await?;

        let (id,): (Uuid,) = sqlx::query_as("SELECT id FROM event_set WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        let event_set_id = EventSetId::from(id);

        let event_set = event_set_repo.find_by_id(&user_id, &event_set_id).await?;
        assert!(event_set.is_some());
        let event_set = event_set.unwrap();
        assert_eq!(event_set.id, event_set_id);
        assert_eq!(event_set.operation, EventSetOperation::CreateAuthor);

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_id_returns_none_for_wrong_user(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let author_repo = PgAuthorRepository::new(pool.clone());
        let event_set_repo = PgEventSetRepository::new(pool.clone());

        let user1_id = prepare_user(&user_repo, "user1").await?;
        let user2_id = prepare_user(&user_repo, "user2").await?;
        let author_id = AuthorId::try_from("278935cf-ed83-4346-9b35-b84bbdb630c0")?;
        create_author(
            &pool,
            &author_repo,
            &user1_id,
            &Author::new(author_id, AuthorName::new("author1".to_owned())?)?,
        )
        .await?;

        let (id,): (Uuid,) = sqlx::query_as("SELECT id FROM event_set WHERE user_id = $1")
            .bind(user1_id.as_str())
            .fetch_one(&pool)
            .await?;
        let event_set_id = EventSetId::from(id);

        // user2 must not see user1's event set.
        let event_set = event_set_repo.find_by_id(&user2_id, &event_set_id).await?;
        assert!(event_set.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn find_by_id_returns_none_for_unknown_id(pool: PgPool) -> anyhow::Result<()> {
        let user_repo = PgUserRepository::new(pool.clone());
        let event_set_repo = PgEventSetRepository::new(pool.clone());

        let user_id = prepare_user(&user_repo, "user1").await?;
        let unknown = EventSetId::try_from("00000000-0000-0000-0000-000000000000")
            .map_err(DomainError::Unexpected)?;

        let event_set = event_set_repo.find_by_id(&user_id, &unknown).await?;
        assert!(event_set.is_none());

        Ok(())
    }
}
