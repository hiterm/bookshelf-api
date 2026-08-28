use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::{AuthorId, AuthorName},
            book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            operation::{Operation, OperationDetail, OperationId, OperationType},
            revision::{
                AuthorOperationChange, AuthorRevision, BookOperationChange, BookRevision,
                RevisionNumber,
            },
            user::UserId,
        },
        error::DomainError,
        repository::history_repository::HistoryRepository,
    },
};

#[derive(sqlx::FromRow)]
struct OperationRow {
    id: Uuid,
    user_id: String,
    r#type: String,
    detail: Option<serde_json::Value>,
    undo_of_operation_id: Option<Uuid>,
    created_at: OffsetDateTime,
}

impl TryFrom<OperationRow> for Operation {
    type Error = DomainError;

    fn try_from(row: OperationRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: OperationId::from(row.id),
            user_id: UserId::new(row.user_id)?,
            operation_type: OperationType::try_from(row.r#type.as_str())
                .map_err(DomainError::Unexpected)?,
            detail: row
                .detail
                .map(serde_json::from_value::<OperationDetail>)
                .transpose()
                .map_err(|error| DomainError::Unexpected(error.to_string()))?,
            undo_of_operation_id: row.undo_of_operation_id.map(OperationId::from),
            created_at: row.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct BookRevisionRow {
    book_id: Uuid,
    revision_number: i32,
    user_id: String,
    title: String,
    author_ids: Option<Vec<Uuid>>,
    isbn: String,
    read: bool,
    owned: bool,
    priority: i32,
    format: String,
    store: String,
    book_created_at: OffsetDateTime,
    book_updated_at: OffsetDateTime,
    created_at: OffsetDateTime,
}

impl TryFrom<BookRevisionRow> for BookRevision {
    type Error = DomainError;

    fn try_from(row: BookRevisionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            book_id: BookId::new(row.book_id)?,
            revision_number: RevisionNumber::try_from(row.revision_number)?,
            user_id: UserId::new(row.user_id)?,
            title: BookTitle::new(row.title)?,
            author_ids: row
                .author_ids
                .unwrap_or_default()
                .into_iter()
                .map(AuthorId::new)
                .collect(),
            isbn: Isbn::new(row.isbn)?,
            read: ReadFlag::new(row.read),
            owned: OwnedFlag::new(row.owned),
            priority: Priority::new(row.priority)?,
            format: BookFormat::try_from(row.format.as_str())
                .map_err(|error| DomainError::Unexpected(error.to_string()))?,
            store: BookStore::try_from(row.store.as_str())
                .map_err(|error| DomainError::Unexpected(error.to_string()))?,
            book_created_at: row.book_created_at,
            book_updated_at: row.book_updated_at,
            created_at: row.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct AuthorRevisionRow {
    author_id: Uuid,
    revision_number: i32,
    user_id: String,
    name: String,
    yomi: String,
    author_created_at: OffsetDateTime,
    author_updated_at: OffsetDateTime,
    created_at: OffsetDateTime,
}

impl TryFrom<AuthorRevisionRow> for AuthorRevision {
    type Error = DomainError;

    fn try_from(row: AuthorRevisionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            author_id: AuthorId::new(row.author_id),
            revision_number: RevisionNumber::try_from(row.revision_number)?,
            user_id: UserId::new(row.user_id)?,
            name: AuthorName::new(row.name)?,
            yomi: row.yomi,
            author_created_at: row.author_created_at,
            author_updated_at: row.author_updated_at,
            created_at: row.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct BookChangeRow {
    operation_id: Uuid,
    book_id: Uuid,
    before_revision_number: Option<i32>,
    after_revision_number: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct AuthorChangeRow {
    operation_id: Uuid,
    author_id: Uuid,
    before_revision_number: Option<i32>,
    after_revision_number: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct PgHistoryRepository {
    pool: PgPool,
}

impl PgHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HistoryRepository for PgHistoryRepository {
    async fn find_operations(&self, user_id: &UserId) -> Result<Vec<Operation>, DomainError> {
        let rows: Vec<OperationRow> = sqlx::query_as(
            "SELECT id, user_id, type, detail, undo_of_operation_id, created_at
             FROM operation
             WHERE user_id = $1 AND type <> 'baseline'
             ORDER BY created_at DESC, id DESC",
        )
        .bind(user_id.as_str())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Operation::try_from).collect()
    }

    async fn find_operation(
        &self,
        user_id: &UserId,
        operation_id: &OperationId,
    ) -> Result<Option<Operation>, DomainError> {
        sqlx::query_as::<_, OperationRow>(
            "SELECT id, user_id, type, detail, undo_of_operation_id, created_at
             FROM operation WHERE user_id = $1 AND id = $2",
        )
        .bind(user_id.as_str())
        .bind(operation_id.to_uuid())
        .fetch_optional(&self.pool)
        .await?
        .map(Operation::try_from)
        .transpose()
    }

    async fn find_book_revisions(
        &self,
        user_id: &UserId,
        book_id: &BookId,
    ) -> Result<Vec<BookRevision>, DomainError> {
        let rows: Vec<BookRevisionRow> = sqlx::query_as(
            "SELECT revision.book_id, revision.revision_number, revision.user_id,
                    revision.title,
                    (SELECT array_agg(link.author_id ORDER BY link.author_id)
                     FROM book_revision_author link
                     WHERE link.user_id = revision.user_id
                       AND link.book_id = revision.book_id
                       AND link.revision_number = revision.revision_number) AS author_ids,
                    revision.isbn, revision.read, revision.owned, revision.priority,
                    revision.format, revision.store, revision.book_created_at,
                    revision.book_updated_at, revision.created_at
             FROM book_revision revision
             WHERE revision.user_id = $1 AND revision.book_id = $2
             ORDER BY revision.revision_number DESC",
        )
        .bind(user_id.as_str())
        .bind(book_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(BookRevision::try_from).collect()
    }

    async fn find_book_revision(
        &self,
        user_id: &UserId,
        book_id: &BookId,
        revision_number: RevisionNumber,
    ) -> Result<Option<BookRevision>, DomainError> {
        sqlx::query_as::<_, BookRevisionRow>(
            "SELECT revision.book_id, revision.revision_number, revision.user_id,
                    revision.title,
                    (SELECT array_agg(link.author_id ORDER BY link.author_id)
                     FROM book_revision_author link
                     WHERE link.user_id = revision.user_id
                       AND link.book_id = revision.book_id
                       AND link.revision_number = revision.revision_number) AS author_ids,
                    revision.isbn, revision.read, revision.owned, revision.priority,
                    revision.format, revision.store, revision.book_created_at,
                    revision.book_updated_at, revision.created_at
             FROM book_revision revision
             WHERE revision.user_id = $1 AND revision.book_id = $2
               AND revision.revision_number = $3",
        )
        .bind(user_id.as_str())
        .bind(book_id.to_uuid())
        .bind(revision_number.value())
        .fetch_optional(&self.pool)
        .await?
        .map(BookRevision::try_from)
        .transpose()
    }

    async fn find_author_revisions(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
    ) -> Result<Vec<AuthorRevision>, DomainError> {
        let rows: Vec<AuthorRevisionRow> = sqlx::query_as(
            "SELECT author_id, revision_number, user_id, name, yomi,
                    author_created_at, author_updated_at, created_at
             FROM author_revision
             WHERE user_id = $1 AND author_id = $2
             ORDER BY revision_number DESC",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(AuthorRevision::try_from).collect()
    }

    async fn find_author_revision(
        &self,
        user_id: &UserId,
        author_id: &AuthorId,
        revision_number: RevisionNumber,
    ) -> Result<Option<AuthorRevision>, DomainError> {
        sqlx::query_as::<_, AuthorRevisionRow>(
            "SELECT author_id, revision_number, user_id, name, yomi,
                    author_created_at, author_updated_at, created_at
             FROM author_revision
             WHERE user_id = $1 AND author_id = $2 AND revision_number = $3",
        )
        .bind(user_id.as_str())
        .bind(author_id.to_uuid())
        .bind(revision_number.value())
        .fetch_optional(&self.pool)
        .await?
        .map(AuthorRevision::try_from)
        .transpose()
    }

    async fn find_book_changes_by_operation_ids(
        &self,
        user_id: &UserId,
        operation_ids: &[OperationId],
    ) -> Result<HashMap<OperationId, Vec<BookOperationChange>>, DomainError> {
        if operation_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let ids: Vec<_> = operation_ids.iter().map(OperationId::to_uuid).collect();
        let rows: Vec<BookChangeRow> = sqlx::query_as(
            "SELECT operation_id, book_id, before_revision_number,
                    after_revision_number
             FROM book_operation_change
             WHERE user_id = $1 AND operation_id = ANY($2::uuid[])
             ORDER BY operation_id, book_id",
        )
        .bind(user_id.as_str())
        .bind(&ids)
        .fetch_all(&self.pool)
        .await?;
        let mut result = operation_ids
            .iter()
            .cloned()
            .map(|id| (id, Vec::new()))
            .collect::<HashMap<_, _>>();
        for row in rows {
            let operation_id = OperationId::from(row.operation_id);
            result
                .entry(operation_id.clone())
                .or_default()
                .push(BookOperationChange {
                    operation_id,
                    book_id: BookId::new(row.book_id)?,
                    before_revision_number: row
                        .before_revision_number
                        .map(RevisionNumber::try_from)
                        .transpose()?,
                    after_revision_number: row
                        .after_revision_number
                        .map(RevisionNumber::try_from)
                        .transpose()?,
                });
        }
        Ok(result)
    }

    async fn find_author_changes_by_operation_ids(
        &self,
        user_id: &UserId,
        operation_ids: &[OperationId],
    ) -> Result<HashMap<OperationId, Vec<AuthorOperationChange>>, DomainError> {
        if operation_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let ids: Vec<_> = operation_ids.iter().map(OperationId::to_uuid).collect();
        let rows: Vec<AuthorChangeRow> = sqlx::query_as(
            "SELECT operation_id, author_id, before_revision_number,
                    after_revision_number
             FROM author_operation_change
             WHERE user_id = $1 AND operation_id = ANY($2::uuid[])
             ORDER BY operation_id, author_id",
        )
        .bind(user_id.as_str())
        .bind(&ids)
        .fetch_all(&self.pool)
        .await?;
        let mut result = operation_ids
            .iter()
            .cloned()
            .map(|id| (id, Vec::new()))
            .collect::<HashMap<_, _>>();
        for row in rows {
            let operation_id = OperationId::from(row.operation_id);
            result
                .entry(operation_id.clone())
                .or_default()
                .push(AuthorOperationChange {
                    operation_id,
                    author_id: AuthorId::new(row.author_id),
                    before_revision_number: row
                        .before_revision_number
                        .map(RevisionNumber::try_from)
                        .transpose()?,
                    after_revision_number: row
                        .after_revision_number
                        .map(RevisionNumber::try_from)
                        .transpose()?,
                });
        }
        Ok(result)
    }
}

#[cfg(all(test, feature = "test-with-database"))]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::domain::{
        entity::{
            author::AuthorId,
            book::BookId,
            operation::{OperationDetail, OperationId, OperationType},
            revision::RevisionNumber,
            user::{User, UserId},
        },
        repository::{history_repository::HistoryRepository, user_repository::UserRepository},
    };
    use crate::infrastructure::{
        history_repository::PgHistoryRepository, user_repository::PgUserRepository,
    };

    async fn user(pool: &PgPool, value: &str) -> anyhow::Result<UserId> {
        let user_id = UserId::new(value.to_owned())?;
        PgUserRepository::new(pool.clone())
            .create(&User::new(user_id.clone()))
            .await?;
        Ok(user_id)
    }

    #[sqlx::test]
    async fn operations_are_owned_typed_and_hide_baseline(pool: PgPool) -> anyhow::Result<()> {
        let owner = user(&pool, "history-owner").await?;
        let other = user(&pool, "history-other").await?;
        let baseline_id = Uuid::new_v4();
        let operation_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO operation (id, user_id, type) VALUES
             ($1, $2, 'baseline'), ($3, $2, 'import_books'),
             ($4, $5, 'create_book')",
        )
        .bind(baseline_id)
        .bind(owner.as_str())
        .bind(operation_id)
        .bind(Uuid::new_v4())
        .bind(other.as_str())
        .execute(&pool)
        .await?;
        sqlx::query("UPDATE operation SET detail = $1 WHERE id = $2")
            .bind(serde_json::json!({"type": "import_books", "imported_count": 2}))
            .bind(operation_id)
            .execute(&pool)
            .await?;

        let repository = PgHistoryRepository::new(pool);
        let operations = repository.find_operations(&owner).await?;
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].operation_type, OperationType::ImportBooks);
        assert_eq!(
            operations[0].detail,
            Some(OperationDetail::ImportBooks { imported_count: 2 })
        );
        assert!(
            repository
                .find_operation(&owner, &OperationId::from(baseline_id))
                .await?
                .is_some()
        );
        assert!(
            repository
                .find_operation(&other, &OperationId::from(operation_id))
                .await?
                .is_none()
        );
        Ok(())
    }

    #[sqlx::test]
    async fn revisions_and_changes_are_owned_and_batched(pool: PgPool) -> anyhow::Result<()> {
        let owner = user(&pool, "revision-owner").await?;
        let other = user(&pool, "revision-other").await?;
        let operation_id = Uuid::new_v4();
        let book_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        sqlx::query("INSERT INTO operation (id, user_id, type) VALUES ($1, $2, 'create_book')")
            .bind(operation_id)
            .bind(owner.as_str())
            .execute(&pool)
            .await?;
        sqlx::query(
            "INSERT INTO author_revision (
               author_id, revision_number, user_id, name, yomi,
               author_created_at, author_updated_at
             ) VALUES ($1, 1, $2, 'Author', '', current_timestamp, current_timestamp)",
        )
        .bind(author_id)
        .bind(owner.as_str())
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO author_revision (
               author_id, revision_number, user_id, name, yomi,
               author_created_at, author_updated_at
             ) VALUES
               ($1, 2, $2, 'Author Two', '', current_timestamp, current_timestamp),
               ($1, 7, $3, 'Other Author', '', current_timestamp, current_timestamp)",
        )
        .bind(author_id)
        .bind(owner.as_str())
        .bind(other.as_str())
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO book_revision (
               book_id, revision_number, user_id, title, isbn, read, owned,
               priority, format, store, book_created_at, book_updated_at
             ) VALUES ($1, 1, $2, 'Book', '', false, true, 4, 'Printed',
                       'Unknown', current_timestamp, current_timestamp)",
        )
        .bind(book_id)
        .bind(owner.as_str())
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO book_revision (
               book_id, revision_number, user_id, title, isbn, read, owned,
               priority, format, store, book_created_at, book_updated_at
             ) VALUES
               ($1, 2, $2, 'Book Two', '', false, true, 4, 'Printed',
                'Unknown', current_timestamp, current_timestamp),
               ($1, 7, $3, 'Other Book', '', false, true, 4, 'Printed',
                'Unknown', current_timestamp, current_timestamp)",
        )
        .bind(book_id)
        .bind(owner.as_str())
        .bind(other.as_str())
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO book_revision_author
               (user_id, book_id, revision_number, author_id)
             VALUES ($1, $2, 1, $3)",
        )
        .bind(owner.as_str())
        .bind(book_id)
        .bind(author_id)
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO book_operation_change
               (operation_id, user_id, book_id, after_revision_number)
             VALUES ($1, $2, $3, 1)",
        )
        .bind(operation_id)
        .bind(owner.as_str())
        .bind(book_id)
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO author_operation_change
               (operation_id, user_id, author_id, after_revision_number)
             VALUES ($1, $2, $3, 1)",
        )
        .bind(operation_id)
        .bind(owner.as_str())
        .bind(author_id)
        .execute(&pool)
        .await?;

        let repository = PgHistoryRepository::new(pool);
        let book_id = BookId::new(book_id)?;
        let author_id = AuthorId::new(author_id);
        let operation_id = OperationId::from(operation_id);
        let book_revision = repository
            .find_book_revision(&owner, &book_id, RevisionNumber::FIRST)
            .await?
            .expect("owned Book revision");
        assert_eq!(book_revision.title.as_str(), "Book");
        assert_eq!(book_revision.author_ids, vec![author_id.clone()]);
        let book_revisions = repository.find_book_revisions(&owner, &book_id).await?;
        assert_eq!(
            book_revisions
                .iter()
                .map(|revision| revision.revision_number.value())
                .collect::<Vec<_>>(),
            vec![2, 1]
        );
        assert!(
            repository
                .find_book_revision(&owner, &book_id, RevisionNumber::try_from(7)?)
                .await?
                .is_none()
        );
        let author_revisions = repository.find_author_revisions(&owner, &author_id).await?;
        assert_eq!(author_revisions.len(), 2);
        let author_revision = repository
            .find_author_revision(&owner, &author_id, RevisionNumber::try_from(2)?)
            .await?
            .expect("requested owned Author revision");
        assert_eq!(author_revision.name.as_str(), "Author Two");
        assert!(
            repository
                .find_author_revision(&owner, &author_id, RevisionNumber::try_from(7)?)
                .await?
                .is_none()
        );

        let book_changes = repository
            .find_book_changes_by_operation_ids(&owner, std::slice::from_ref(&operation_id))
            .await?;
        assert_eq!(book_changes[&operation_id].len(), 1);
        let author_changes = repository
            .find_author_changes_by_operation_ids(&owner, std::slice::from_ref(&operation_id))
            .await?;
        assert_eq!(author_changes[&operation_id].len(), 1);
        Ok(())
    }
}
