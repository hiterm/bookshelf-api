use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::{
    entity::{operation::OperationId, user::UserId},
    error::DomainError,
};

#[derive(Debug, FromRow)]
struct BookChange {
    book_id: Uuid,
    before_revision_number: Option<i32>,
    after_revision_number: Option<i32>,
}

#[derive(Debug, FromRow)]
struct AuthorChange {
    author_id: Uuid,
    before_revision_number: Option<i32>,
    after_revision_number: Option<i32>,
}

pub async fn undo_operation(
    pool: &PgPool,
    user_id: &UserId,
    target_id: &OperationId,
) -> Result<OperationId, DomainError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *tx)
        .await?;
    let operation_type = sqlx::query_scalar::<_, String>(
        "SELECT type FROM operation
         WHERE user_id = $1 AND id = $2
         FOR UPDATE",
    )
    .bind(user_id.as_str())
    .bind(target_id.to_uuid())
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| DomainError::NotFound {
        entity_type: "operation",
        entity_id: target_id.to_string(),
        user_id: user_id.as_str().to_owned(),
    })?;
    if operation_type == "baseline" {
        return Err(DomainError::Conflict(
            "baseline operations cannot be undone".to_owned(),
        ));
    }

    let book_changes: Vec<BookChange> = sqlx::query_as(
        "SELECT book_id, before_revision_number, after_revision_number
         FROM book_operation_change
         WHERE user_id = $1 AND operation_id = $2
         ORDER BY book_id",
    )
    .bind(user_id.as_str())
    .bind(target_id.to_uuid())
    .fetch_all(&mut *tx)
    .await?;
    let author_changes: Vec<AuthorChange> = sqlx::query_as(
        "SELECT author_id, before_revision_number, after_revision_number
         FROM author_operation_change
         WHERE user_id = $1 AND operation_id = $2
         ORDER BY author_id",
    )
    .bind(user_id.as_str())
    .bind(target_id.to_uuid())
    .fetch_all(&mut *tx)
    .await?;
    if book_changes.is_empty() && author_changes.is_empty() {
        return Err(DomainError::Conflict(
            "operation has no entity changes to undo".to_owned(),
        ));
    }

    let mut lock_keys = book_changes
        .iter()
        .map(|change| format!("book:{}:{}", user_id.as_str(), change.book_id))
        .chain(
            author_changes
                .iter()
                .map(|change| format!("author:{}:{}", user_id.as_str(), change.author_id)),
        )
        .collect::<Vec<_>>();
    lock_keys.sort_unstable();
    for key in lock_keys {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(key)
            .execute(&mut *tx)
            .await?;
    }

    let book_ids = book_changes
        .iter()
        .map(|change| change.book_id)
        .collect::<Vec<_>>();
    sqlx::query(
        "SELECT id FROM book
         WHERE user_id = $1 AND id = ANY($2::uuid[])
         ORDER BY id FOR UPDATE",
    )
    .bind(user_id.as_str())
    .bind(&book_ids)
    .fetch_all(&mut *tx)
    .await?;
    let author_ids = author_changes
        .iter()
        .map(|change| change.author_id)
        .collect::<Vec<_>>();
    sqlx::query(
        "SELECT id FROM author
         WHERE user_id = $1 AND id = ANY($2::uuid[])
         ORDER BY id FOR UPDATE",
    )
    .bind(user_id.as_str())
    .bind(&author_ids)
    .fetch_all(&mut *tx)
    .await?;

    if !matches_after_state(&mut tx, user_id, &book_changes, &author_changes).await? {
        return Err(DomainError::Conflict(
            "operation is no longer undoable because an affected entity changed".to_owned(),
        ));
    }

    let undo_id = OperationId::new();
    sqlx::query(
        "INSERT INTO operation (id, user_id, type, undo_of_operation_id)
         VALUES ($1, $2, 'undo', $3)",
    )
    .bind(undo_id.to_uuid())
    .bind(user_id.as_str())
    .bind(target_id.to_uuid())
    .execute(&mut *tx)
    .await?;

    for change in author_changes
        .iter()
        .filter(|change| change.before_revision_number.is_some())
    {
        restore_author(&mut tx, user_id, &undo_id, change).await?;
    }
    for change in book_changes
        .iter()
        .filter(|change| change.before_revision_number.is_some())
    {
        restore_book(&mut tx, user_id, &undo_id, change).await?;
    }
    for change in book_changes
        .iter()
        .filter(|change| change.before_revision_number.is_none())
    {
        delete_book(&mut tx, user_id, &undo_id, change).await?;
    }
    for change in author_changes
        .iter()
        .filter(|change| change.before_revision_number.is_none())
    {
        delete_author(&mut tx, user_id, &undo_id, change).await?;
    }

    tx.commit().await?;
    Ok(undo_id)
}

async fn matches_after_state(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    books: &[BookChange],
    authors: &[AuthorChange],
) -> Result<bool, DomainError> {
    for change in books {
        let current = sqlx::query_scalar::<_, i32>(
            "SELECT (SELECT MAX(revision_number) FROM book_revision
                     WHERE user_id = $1 AND book_id = $2)
             WHERE EXISTS (SELECT 1 FROM book WHERE user_id = $1 AND id = $2)",
        )
        .bind(user_id.as_str())
        .bind(change.book_id)
        .fetch_optional(&mut **tx)
        .await?;
        if current != change.after_revision_number {
            return Ok(false);
        }
    }
    for change in authors {
        let current = sqlx::query_scalar::<_, i32>(
            "SELECT (SELECT MAX(revision_number) FROM author_revision
                     WHERE user_id = $1 AND author_id = $2)
             WHERE EXISTS (SELECT 1 FROM author WHERE user_id = $1 AND id = $2)",
        )
        .bind(user_id.as_str())
        .bind(change.author_id)
        .fetch_optional(&mut **tx)
        .await?;
        if current != change.after_revision_number {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn restore_author(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    undo_id: &OperationId,
    change: &AuthorChange,
) -> Result<(), DomainError> {
    let source = change.before_revision_number.expect("filtered revision");
    let next = next_author_revision(tx, user_id, change.author_id).await?;
    sqlx::query(
        "INSERT INTO author (id, user_id, name, yomi, created_at, updated_at)
         SELECT author_id, user_id, name, yomi, author_created_at, current_timestamp
         FROM author_revision
         WHERE user_id = $1 AND author_id = $2 AND revision_number = $3
         ON CONFLICT (id, user_id) DO UPDATE
         SET name = EXCLUDED.name, yomi = EXCLUDED.yomi,
             created_at = EXCLUDED.created_at, updated_at = EXCLUDED.updated_at",
    )
    .bind(user_id.as_str())
    .bind(change.author_id)
    .bind(source)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "INSERT INTO author_revision (
           author_id, revision_number, user_id, name, yomi,
           author_created_at, author_updated_at
         ) SELECT author_id, $4, user_id, name, yomi,
                  author_created_at, current_timestamp
           FROM author_revision
           WHERE user_id = $1 AND author_id = $2 AND revision_number = $3",
    )
    .bind(user_id.as_str())
    .bind(change.author_id)
    .bind(source)
    .bind(next)
    .execute(&mut **tx)
    .await?;
    insert_author_change(
        tx,
        user_id,
        undo_id,
        change.author_id,
        change.after_revision_number,
        Some(next),
    )
    .await
}

async fn restore_book(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    undo_id: &OperationId,
    change: &BookChange,
) -> Result<(), DomainError> {
    let source = change.before_revision_number.expect("filtered revision");
    let missing_author = sqlx::query_scalar::<_, Uuid>(
        "SELECT link.author_id
         FROM book_revision_author link
         LEFT JOIN author current
           ON current.user_id = link.user_id AND current.id = link.author_id
         WHERE link.user_id = $1 AND link.book_id = $2
           AND link.revision_number = $3 AND current.id IS NULL
         LIMIT 1",
    )
    .bind(user_id.as_str())
    .bind(change.book_id)
    .bind(source)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(author_id) = missing_author {
        return Err(DomainError::Conflict(format!(
            "book revision references missing author {author_id}"
        )));
    }
    let next = next_book_revision(tx, user_id, change.book_id).await?;
    sqlx::query(
        "INSERT INTO book (
           id, user_id, title, isbn, read, owned, priority, format, store,
           created_at, updated_at
         ) SELECT book_id, user_id, title, isbn, read, owned, priority, format,
                  store, book_created_at, current_timestamp
           FROM book_revision
           WHERE user_id = $1 AND book_id = $2 AND revision_number = $3
         ON CONFLICT (id, user_id) DO UPDATE
         SET title = EXCLUDED.title, isbn = EXCLUDED.isbn, read = EXCLUDED.read,
             owned = EXCLUDED.owned, priority = EXCLUDED.priority,
             format = EXCLUDED.format, store = EXCLUDED.store,
             created_at = EXCLUDED.created_at, updated_at = EXCLUDED.updated_at",
    )
    .bind(user_id.as_str())
    .bind(change.book_id)
    .bind(source)
    .execute(&mut **tx)
    .await?;
    sqlx::query("DELETE FROM book_author WHERE user_id = $1 AND book_id = $2")
        .bind(user_id.as_str())
        .bind(change.book_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "INSERT INTO book_author (user_id, book_id, author_id)
         SELECT user_id, book_id, author_id FROM book_revision_author
         WHERE user_id = $1 AND book_id = $2 AND revision_number = $3",
    )
    .bind(user_id.as_str())
    .bind(change.book_id)
    .bind(source)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "INSERT INTO book_revision (
           book_id, revision_number, user_id, title, isbn, read, owned,
           priority, format, store, book_created_at, book_updated_at
         ) SELECT book_id, $4, user_id, title, isbn, read, owned, priority,
                  format, store, book_created_at, current_timestamp
           FROM book_revision
           WHERE user_id = $1 AND book_id = $2 AND revision_number = $3",
    )
    .bind(user_id.as_str())
    .bind(change.book_id)
    .bind(source)
    .bind(next)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "INSERT INTO book_revision_author (user_id, book_id, revision_number, author_id)
         SELECT user_id, book_id, $4, author_id FROM book_revision_author
         WHERE user_id = $1 AND book_id = $2 AND revision_number = $3",
    )
    .bind(user_id.as_str())
    .bind(change.book_id)
    .bind(source)
    .bind(next)
    .execute(&mut **tx)
    .await?;
    insert_book_change(
        tx,
        user_id,
        undo_id,
        change.book_id,
        change.after_revision_number,
        Some(next),
    )
    .await
}

async fn delete_book(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    undo_id: &OperationId,
    change: &BookChange,
) -> Result<(), DomainError> {
    sqlx::query("DELETE FROM book_author WHERE user_id = $1 AND book_id = $2")
        .bind(user_id.as_str())
        .bind(change.book_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM book WHERE user_id = $1 AND id = $2")
        .bind(user_id.as_str())
        .bind(change.book_id)
        .execute(&mut **tx)
        .await?;
    insert_book_change(
        tx,
        user_id,
        undo_id,
        change.book_id,
        change.after_revision_number,
        None,
    )
    .await
}

async fn delete_author(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    undo_id: &OperationId,
    change: &AuthorChange,
) -> Result<(), DomainError> {
    sqlx::query("DELETE FROM author WHERE user_id = $1 AND id = $2")
        .bind(user_id.as_str())
        .bind(change.author_id)
        .execute(&mut **tx)
        .await?;
    insert_author_change(
        tx,
        user_id,
        undo_id,
        change.author_id,
        change.after_revision_number,
        None,
    )
    .await
}

async fn next_book_revision(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    id: Uuid,
) -> Result<i32, DomainError> {
    Ok(sqlx::query_scalar(
        "SELECT COALESCE(MAX(revision_number), 0) + 1
         FROM book_revision WHERE user_id = $1 AND book_id = $2",
    )
    .bind(user_id.as_str())
    .bind(id)
    .fetch_one(&mut **tx)
    .await?)
}

async fn next_author_revision(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    id: Uuid,
) -> Result<i32, DomainError> {
    Ok(sqlx::query_scalar(
        "SELECT COALESCE(MAX(revision_number), 0) + 1
         FROM author_revision WHERE user_id = $1 AND author_id = $2",
    )
    .bind(user_id.as_str())
    .bind(id)
    .fetch_one(&mut **tx)
    .await?)
}

async fn insert_book_change(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    undo_id: &OperationId,
    id: Uuid,
    before: Option<i32>,
    after: Option<i32>,
) -> Result<(), DomainError> {
    sqlx::query("INSERT INTO book_operation_change (operation_id, user_id, book_id, before_revision_number, after_revision_number) VALUES ($1, $2, $3, $4, $5)")
        .bind(undo_id.to_uuid()).bind(user_id.as_str()).bind(id).bind(before).bind(after).execute(&mut **tx).await?;
    Ok(())
}

async fn insert_author_change(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    undo_id: &OperationId,
    id: Uuid,
    before: Option<i32>,
    after: Option<i32>,
) -> Result<(), DomainError> {
    sqlx::query("INSERT INTO author_operation_change (operation_id, user_id, author_id, before_revision_number, after_revision_number) VALUES ($1, $2, $3, $4, $5)")
        .bind(undo_id.to_uuid()).bind(user_id.as_str()).bind(id).bind(before).bind(after).execute(&mut **tx).await?;
    Ok(())
}

#[cfg(all(test, feature = "test-with-database"))]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        domain::{
            entity::{
                operation::{OperationId, OperationType},
                user::{User, UserId},
            },
            repository::{history_repository::HistoryRepository, user_repository::UserRepository},
        },
        infrastructure::{
            history_repository::PgHistoryRepository, user_repository::PgUserRepository,
        },
    };

    async fn owner(pool: &PgPool) -> anyhow::Result<UserId> {
        let user_id = UserId::new("undo-owner".to_owned())?;
        PgUserRepository::new(pool.clone())
            .create(&User::new(user_id.clone()))
            .await?;
        Ok(user_id)
    }

    async fn insert_book_revision(
        pool: &PgPool,
        user_id: &UserId,
        book_id: Uuid,
        revision: i32,
        title: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO book_revision (
               book_id, revision_number, user_id, title, isbn, read, owned,
               priority, format, store, book_created_at, book_updated_at
             ) VALUES ($1, $2, $3, $4, '', false, true, 4, 'Printed',
                       'Unknown', current_timestamp, current_timestamp)",
        )
        .bind(book_id)
        .bind(revision)
        .bind(user_id.as_str())
        .bind(title)
        .execute(pool)
        .await?;
        Ok(())
    }

    #[sqlx::test]
    async fn undo_create_deletes_current_book_and_records_inverse(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_id = owner(&pool).await?;
        let target_id = Uuid::new_v4();
        let book_id = Uuid::new_v4();
        sqlx::query("INSERT INTO operation (id, user_id, type) VALUES ($1, $2, 'create_book')")
            .bind(target_id)
            .bind(user_id.as_str())
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store) VALUES ($1, $2, 'Book', '', false, true, 4, 'Printed', 'Unknown')")
            .bind(book_id).bind(user_id.as_str()).execute(&pool).await?;
        insert_book_revision(&pool, &user_id, book_id, 1, "Book").await?;
        sqlx::query("INSERT INTO book_operation_change (operation_id, user_id, book_id, after_revision_number) VALUES ($1, $2, $3, 1)")
            .bind(target_id).bind(user_id.as_str()).bind(book_id).execute(&pool).await?;

        let repository = PgHistoryRepository::new(pool.clone());
        let undo_id = repository
            .undo_operation(&user_id, &OperationId::from(target_id))
            .await?;

        let current: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM book WHERE user_id = $1 AND id = $2)")
                .bind(user_id.as_str())
                .bind(book_id)
                .fetch_one(&pool)
                .await?;
        assert!(!current);
        let undo = repository
            .find_operation(&user_id, &undo_id)
            .await?
            .expect("recorded undo");
        assert_eq!(undo.operation_type, OperationType::Undo);
        assert_eq!(
            undo.undo_of_operation_id,
            Some(OperationId::from(target_id))
        );
        let inverse: (Option<i32>, Option<i32>) = sqlx::query_as(
            "SELECT before_revision_number, after_revision_number
             FROM book_operation_change WHERE operation_id = $1 AND book_id = $2",
        )
        .bind(undo_id.to_uuid())
        .bind(book_id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(inverse, (Some(1), None));
        Ok(())
    }

    #[sqlx::test]
    async fn undo_update_restores_snapshot_as_fresh_revision(pool: PgPool) -> anyhow::Result<()> {
        let user_id = owner(&pool).await?;
        let target_id = Uuid::new_v4();
        let book_id = Uuid::new_v4();
        sqlx::query("INSERT INTO operation (id, user_id, type) VALUES ($1, $2, 'update_book')")
            .bind(target_id)
            .bind(user_id.as_str())
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store) VALUES ($1, $2, 'New', '', false, true, 4, 'Printed', 'Unknown')")
            .bind(book_id).bind(user_id.as_str()).execute(&pool).await?;
        insert_book_revision(&pool, &user_id, book_id, 1, "Old").await?;
        insert_book_revision(&pool, &user_id, book_id, 2, "New").await?;
        sqlx::query("INSERT INTO book_operation_change (operation_id, user_id, book_id, before_revision_number, after_revision_number) VALUES ($1, $2, $3, 1, 2)")
            .bind(target_id).bind(user_id.as_str()).bind(book_id).execute(&pool).await?;

        let repository = PgHistoryRepository::new(pool.clone());
        let undo_id = repository
            .undo_operation(&user_id, &OperationId::from(target_id))
            .await?;

        let title: String =
            sqlx::query_scalar("SELECT title FROM book WHERE user_id = $1 AND id = $2")
                .bind(user_id.as_str())
                .bind(book_id)
                .fetch_one(&pool)
                .await?;
        assert_eq!(title, "Old");
        let revision: (String, i32) = sqlx::query_as(
            "SELECT title, revision_number FROM book_revision
             WHERE user_id = $1 AND book_id = $2 ORDER BY revision_number DESC LIMIT 1",
        )
        .bind(user_id.as_str())
        .bind(book_id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(revision, ("Old".to_owned(), 3));
        let inverse: (Option<i32>, Option<i32>) = sqlx::query_as(
            "SELECT before_revision_number, after_revision_number
             FROM book_operation_change WHERE operation_id = $1 AND book_id = $2",
        )
        .bind(undo_id.to_uuid())
        .bind(book_id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(inverse, (Some(2), Some(3)));
        Ok(())
    }

    #[sqlx::test]
    async fn undo_multi_entity_create_deletes_book_before_author(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_id = owner(&pool).await?;
        let target_id = Uuid::new_v4();
        let book_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        sqlx::query("INSERT INTO operation (id, user_id, type) VALUES ($1, $2, 'import_books')")
            .bind(target_id)
            .bind(user_id.as_str())
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO author (id, user_id, name) VALUES ($1, $2, 'Author')")
            .bind(author_id)
            .bind(user_id.as_str())
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO author_revision (author_id, revision_number, user_id, name, yomi, author_created_at, author_updated_at) VALUES ($1, 1, $2, 'Author', '', current_timestamp, current_timestamp)")
            .bind(author_id).bind(user_id.as_str()).execute(&pool).await?;
        sqlx::query("INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store) VALUES ($1, $2, 'Book', '', false, true, 4, 'Printed', 'Unknown')")
            .bind(book_id).bind(user_id.as_str()).execute(&pool).await?;
        insert_book_revision(&pool, &user_id, book_id, 1, "Book").await?;
        sqlx::query("INSERT INTO book_author (user_id, book_id, author_id) VALUES ($1, $2, $3)")
            .bind(user_id.as_str())
            .bind(book_id)
            .bind(author_id)
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book_revision_author (user_id, book_id, revision_number, author_id) VALUES ($1, $2, 1, $3)")
            .bind(user_id.as_str()).bind(book_id).bind(author_id).execute(&pool).await?;
        sqlx::query("INSERT INTO book_operation_change (operation_id, user_id, book_id, after_revision_number) VALUES ($1, $2, $3, 1)")
            .bind(target_id).bind(user_id.as_str()).bind(book_id).execute(&pool).await?;
        sqlx::query("INSERT INTO author_operation_change (operation_id, user_id, author_id, after_revision_number) VALUES ($1, $2, $3, 1)")
            .bind(target_id).bind(user_id.as_str()).bind(author_id).execute(&pool).await?;

        PgHistoryRepository::new(pool.clone())
            .undo_operation(&user_id, &OperationId::from(target_id))
            .await?;

        let current_count: i64 = sqlx::query_scalar(
            "SELECT (SELECT COUNT(*) FROM book WHERE user_id = $1) +
                    (SELECT COUNT(*) FROM author WHERE user_id = $1)",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(current_count, 0);
        Ok(())
    }

    #[sqlx::test]
    async fn missing_book_author_rolls_back_the_complete_undo(pool: PgPool) -> anyhow::Result<()> {
        let user_id = owner(&pool).await?;
        let target_id = Uuid::new_v4();
        let book_id = Uuid::new_v4();
        let missing_author_id = Uuid::new_v4();
        sqlx::query("INSERT INTO operation (id, user_id, type) VALUES ($1, $2, 'update_book')")
            .bind(target_id)
            .bind(user_id.as_str())
            .execute(&pool)
            .await?;
        sqlx::query("INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store) VALUES ($1, $2, 'Current', '', false, true, 4, 'Printed', 'Unknown')")
            .bind(book_id).bind(user_id.as_str()).execute(&pool).await?;
        insert_book_revision(&pool, &user_id, book_id, 1, "Historical").await?;
        insert_book_revision(&pool, &user_id, book_id, 2, "Current").await?;
        sqlx::query("INSERT INTO book_revision_author (user_id, book_id, revision_number, author_id) VALUES ($1, $2, 1, $3)")
            .bind(user_id.as_str()).bind(book_id).bind(missing_author_id).execute(&pool).await?;
        sqlx::query("INSERT INTO book_operation_change (operation_id, user_id, book_id, before_revision_number, after_revision_number) VALUES ($1, $2, $3, 1, 2)")
            .bind(target_id).bind(user_id.as_str()).bind(book_id).execute(&pool).await?;

        let result = PgHistoryRepository::new(pool.clone())
            .undo_operation(&user_id, &OperationId::from(target_id))
            .await;
        assert!(result.is_err());
        let title: String =
            sqlx::query_scalar("SELECT title FROM book WHERE user_id = $1 AND id = $2")
                .bind(user_id.as_str())
                .bind(book_id)
                .fetch_one(&pool)
                .await?;
        assert_eq!(title, "Current");
        let undo_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM operation WHERE user_id = $1 AND type = 'undo'",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(undo_count, 0);
        Ok(())
    }
}
