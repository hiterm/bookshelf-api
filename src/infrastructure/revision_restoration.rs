use sqlx::PgConnection;
use time::OffsetDateTime;
use uuid::Uuid;

use super::error::classify_author_name_write_error;
use crate::domain::{entity::user::UserId, error::DomainError};

/// Restores the owned current author from an immutable source revision.
/// The caller validates the source name and dependencies and owns the transaction.
/// A supplied timestamp is preserved; Undo uses the transaction's current timestamp.
pub(crate) async fn restore_author_current(
    connection: &mut PgConnection,
    user: &UserId,
    id: Uuid,
    source: i32,
    name: &str,
    updated_at: Option<OffsetDateTime>,
) -> Result<(), DomainError> {
    let result = sqlx::query(
        "INSERT INTO author (id,user_id,name,yomi,created_at,updated_at)
         SELECT author_id,user_id,name,yomi,author_created_at,COALESCE($4,current_timestamp)
         FROM author_revision WHERE user_id=$1 AND author_id=$2 AND revision_number=$3
         ON CONFLICT (id,user_id) DO UPDATE SET
           name=EXCLUDED.name,yomi=EXCLUDED.yomi,
           created_at=EXCLUDED.created_at,updated_at=EXCLUDED.updated_at",
    )
    .bind(user.as_str())
    .bind(id)
    .bind(source)
    .bind(updated_at)
    .execute(connection)
    .await
    .map_err(|error| classify_author_name_write_error(error, name))?;
    require_source(result.rows_affected(), "Author")
}

/// Restores the owned current book and replaces author links from its source snapshot.
/// The caller checks author availability and any domain decoding before invoking this helper.
/// Ordinary restore supplies its entity timestamp; Undo uses the transaction timestamp.
pub(crate) async fn restore_book_current(
    connection: &mut PgConnection,
    user: &UserId,
    id: Uuid,
    source: i32,
    updated_at: Option<OffsetDateTime>,
) -> Result<(), DomainError> {
    let result = sqlx::query(
        "INSERT INTO book (id,user_id,title,isbn,read,owned,priority,format,store,
                           purchase_date,created_at,updated_at)
         SELECT book_id,user_id,title,isbn,read,owned,priority,
                format,store,purchase_date,
                book_created_at,COALESCE($4,current_timestamp)
         FROM book_revision WHERE user_id=$1 AND book_id=$2 AND revision_number=$3
         ON CONFLICT (id,user_id) DO UPDATE SET
           title=EXCLUDED.title,isbn=EXCLUDED.isbn,read=EXCLUDED.read,
           owned=EXCLUDED.owned,priority=EXCLUDED.priority,format=EXCLUDED.format,
           store=EXCLUDED.store,purchase_date=EXCLUDED.purchase_date,
           created_at=EXCLUDED.created_at,updated_at=EXCLUDED.updated_at",
    )
    .bind(user.as_str())
    .bind(id)
    .bind(source)
    .bind(updated_at)
    .execute(&mut *connection)
    .await?;
    require_source(result.rows_affected(), "Book")?;
    sqlx::query("DELETE FROM book_author WHERE user_id=$1 AND book_id=$2")
        .bind(user.as_str())
        .bind(id)
        .execute(&mut *connection)
        .await?;
    sqlx::query(
        "INSERT INTO book_author (user_id,book_id,author_id)
         SELECT user_id,book_id,author_id FROM book_revision_author
         WHERE user_id=$1 AND book_id=$2 AND revision_number=$3",
    )
    .bind(user.as_str())
    .bind(id)
    .bind(source)
    .execute(connection)
    .await?;
    Ok(())
}

/// Ensures a missing source cannot silently clear links or record a misleading inverse.
fn require_source(rows: u64, entity: &str) -> Result<(), DomainError> {
    if rows != 1 {
        return Err(DomainError::Unexpected(format!(
            "{entity} restore source revision is missing"
        )));
    }
    Ok(())
}
