use sqlx::PgConnection;
use uuid::Uuid;

use super::{error::persisted_value, transaction::PgTransaction};
use crate::domain::{
    entity::{operation::OperationId, revision::RevisionNumber, user::UserId},
    error::DomainError,
};

/// Appends the persisted book snapshot and change in the caller's transaction.
/// The repository must have written the entity and author links first.
/// Returns an error for a missing current row, invalid revision, or overflow.
pub async fn append_book_revision(
    tx: &mut PgTransaction,
    id: Uuid,
    before: Option<i32>,
) -> Result<RevisionNumber, DomainError> {
    let user = tx.user_id().clone();
    let operation = tx.operation_id();
    record_book_revision(tx.as_mut(), &user, &operation, id, before).await
}

/// Records a deletion after the repository removes the owned current book.
pub async fn append_book_deletion(
    tx: &mut PgTransaction,
    id: Uuid,
    before: i32,
) -> Result<(), DomainError> {
    let user = tx.user_id().clone();
    let operation = tx.operation_id();
    insert_book_change(tx.as_mut(), &user, &operation, id, Some(before), None).await
}

/// Appends the persisted author snapshot and change in the caller's transaction.
/// The repository must have written the entity first; no transaction is committed here.
pub async fn append_author_revision(
    tx: &mut PgTransaction,
    id: Uuid,
    before: Option<i32>,
) -> Result<RevisionNumber, DomainError> {
    let user = tx.user_id().clone();
    let operation = tx.operation_id();
    record_author_revision(tx.as_mut(), &user, &operation, id, before).await
}

/// Records a deletion after the repository removes the owned current author.
pub async fn append_author_deletion(
    tx: &mut PgTransaction,
    id: Uuid,
    before: i32,
) -> Result<(), DomainError> {
    let user = tx.user_id().clone();
    let operation = tx.operation_id();
    insert_author_change(tx.as_mut(), &user, &operation, id, Some(before), None).await
}

/// Records the current owned book and its author links, preserving its persisted timestamps.
/// The caller provides serialization/locking and owns commit or rollback.
pub(crate) async fn record_book_revision(
    connection: &mut PgConnection,
    user: &UserId,
    operation: &OperationId,
    id: Uuid,
    before: Option<i32>,
) -> Result<RevisionNumber, DomainError> {
    let latest: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(revision_number) FROM book_revision WHERE user_id=$1 AND book_id=$2",
    )
    .bind(user.as_str())
    .bind(id)
    .fetch_one(&mut *connection)
    .await?;
    let next = next_revision(latest, "Book")?;
    let result = sqlx::query(
        "INSERT INTO book_revision (
           book_id, revision_number, user_id, title, isbn, read, owned,
           priority, format, store, purchase_date, book_created_at, book_updated_at
         ) SELECT id, $3, user_id, title, isbn, read, owned, priority, format, store,
                  purchase_date, created_at, updated_at
           FROM book WHERE user_id=$1 AND id=$2",
    )
    .bind(user.as_str())
    .bind(id)
    .bind(next.value())
    .execute(&mut *connection)
    .await?;
    require_current_row(result.rows_affected(), "Book")?;
    sqlx::query(
        "INSERT INTO book_revision_author (user_id,book_id,revision_number,author_id)
         SELECT user_id,book_id,$3,author_id FROM book_author WHERE user_id=$1 AND book_id=$2",
    )
    .bind(user.as_str())
    .bind(id)
    .bind(next.value())
    .execute(&mut *connection)
    .await?;
    insert_book_change(connection, user, operation, id, before, Some(next.value())).await?;
    Ok(next)
}

/// Records the current owned author without decoding or changing its persisted values.
/// The caller provides serialization/locking and owns commit or rollback.
pub(crate) async fn record_author_revision(
    connection: &mut PgConnection,
    user: &UserId,
    operation: &OperationId,
    id: Uuid,
    before: Option<i32>,
) -> Result<RevisionNumber, DomainError> {
    let latest: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(revision_number) FROM author_revision WHERE user_id=$1 AND author_id=$2",
    )
    .bind(user.as_str())
    .bind(id)
    .fetch_one(&mut *connection)
    .await?;
    let next = next_revision(latest, "Author")?;
    let result = sqlx::query(
        "INSERT INTO author_revision (
           author_id, revision_number, user_id, name, yomi, author_created_at, author_updated_at
         ) SELECT id,$3,user_id,name,yomi,created_at,updated_at
           FROM author WHERE user_id=$1 AND id=$2",
    )
    .bind(user.as_str())
    .bind(id)
    .bind(next.value())
    .execute(&mut *connection)
    .await?;
    require_current_row(result.rows_affected(), "Author")?;
    insert_author_change(connection, user, operation, id, before, Some(next.value())).await?;
    Ok(next)
}

/// Allocates a positive revision with checked overflow before any snapshot insert.
fn next_revision(latest: Option<i32>, entity: &str) -> Result<RevisionNumber, DomainError> {
    let next = latest
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| DomainError::Unexpected(format!("{entity} revision number overflow")))?;
    persisted_value("revision number", RevisionNumber::try_from(next))
}

/// Rejects silent INSERT SELECT skips so an operation never records a nonexistent snapshot.
fn require_current_row(rows: u64, entity: &str) -> Result<(), DomainError> {
    if rows != 1 {
        return Err(DomainError::Unexpected(format!(
            "{entity} snapshot requires one owned current row"
        )));
    }
    Ok(())
}

/// Records one owned book change; `None` represents absence before or after.
pub(crate) async fn insert_book_change(
    connection: &mut PgConnection,
    user: &UserId,
    operation: &OperationId,
    id: Uuid,
    before: Option<i32>,
    after: Option<i32>,
) -> Result<(), DomainError> {
    sqlx::query("INSERT INTO book_operation_change (operation_id,user_id,book_id,before_revision_number,after_revision_number) VALUES ($1,$2,$3,$4,$5)")
        .bind(operation.to_uuid()).bind(user.as_str()).bind(id).bind(before).bind(after).execute(connection).await?;
    Ok(())
}

/// Records one owned author change; `None` represents absence before or after.
pub(crate) async fn insert_author_change(
    connection: &mut PgConnection,
    user: &UserId,
    operation: &OperationId,
    id: Uuid,
    before: Option<i32>,
    after: Option<i32>,
) -> Result<(), DomainError> {
    sqlx::query("INSERT INTO author_operation_change (operation_id,user_id,author_id,before_revision_number,after_revision_number) VALUES ($1,$2,$3,$4,$5)")
        .bind(operation.to_uuid()).bind(user.as_str()).bind(id).bind(before).bind(after).execute(connection).await?;
    Ok(())
}

/// Returns the latest recorded book revision for the transaction owner.
/// Returns an error when no revision exists.
pub async fn latest_book_revision_number(
    tx: &mut PgTransaction,
    book_id: uuid::Uuid,
) -> Result<i32, DomainError> {
    let user_id = tx.user_id().clone();
    sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(revision_number)
         FROM book_revision
         WHERE book_id = $1 AND user_id = $2",
    )
    .bind(book_id)
    .bind(user_id.as_str())
    .fetch_one(tx.as_mut())
    .await?
    .ok_or_else(|| DomainError::Unexpected("Book has no current revision".to_owned()))
}

/// Returns the latest recorded author revision for the transaction owner.
/// Returns an error when no revision exists.
pub async fn latest_author_revision_number(
    tx: &mut PgTransaction,
    author_id: uuid::Uuid,
) -> Result<i32, DomainError> {
    let user_id = tx.user_id().clone();
    sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(revision_number)
         FROM author_revision
         WHERE author_id = $1 AND user_id = $2",
    )
    .bind(author_id)
    .bind(user_id.as_str())
    .fetch_one(tx.as_mut())
    .await?
    .ok_or_else(|| DomainError::Unexpected("Author has no current revision".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises initial allocation, increments, overflow, and invalid persisted revision values.
    #[test]
    fn revision_allocation_handles_absence_and_overflow() {
        assert_eq!(next_revision(None, "Book").unwrap().value(), 1);
        assert_eq!(next_revision(Some(42), "Author").unwrap().value(), 43);
        assert!(
            matches!(next_revision(Some(i32::MAX), "Book"), Err(DomainError::Unexpected(message)) if message == "Book revision number overflow")
        );
        assert!(matches!(
            next_revision(Some(-1), "Author"),
            Err(DomainError::Unexpected(_))
        ));
    }
}
