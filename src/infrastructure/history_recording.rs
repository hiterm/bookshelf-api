use sqlx::Row;

use crate::domain::{
    entity::{author::Author, book::Book},
    error::DomainError,
};

use super::transaction::PgTransaction;

pub async fn append_book_revision(
    tx: &mut PgTransaction,
    book: &Book,
    before_revision_number: Option<i32>,
) -> Result<i32, DomainError> {
    let user_id = tx.user_id().clone();
    let latest: Option<i32> = sqlx::query(
        "SELECT MAX(revision_number) AS revision_number
         FROM book_revision
         WHERE book_id = $1 AND user_id = $2",
    )
    .bind(book.id().to_uuid())
    .bind(user_id.as_str())
    .fetch_one(tx.as_mut())
    .await?
    .try_get("revision_number")?;
    let revision_number = latest
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| DomainError::Unexpected("Book revision number overflow".to_owned()))?;

    sqlx::query(
        "INSERT INTO book_revision (
           book_id, revision_number, user_id, title, isbn, read, owned,
           priority, format, store, book_created_at, book_updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(book.id().to_uuid())
    .bind(revision_number)
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
    .execute(tx.as_mut())
    .await?;

    let author_ids: Vec<_> = book
        .author_ids()
        .iter()
        .map(|author_id| author_id.to_uuid())
        .collect();
    if !author_ids.is_empty() {
        sqlx::query(
            "INSERT INTO book_revision_author (user_id, book_id, revision_number, author_id)
             SELECT $1, $2, $3, author_id
             FROM UNNEST($4::uuid[]) AS input(author_id)",
        )
        .bind(user_id.as_str())
        .bind(book.id().to_uuid())
        .bind(revision_number)
        .bind(&author_ids)
        .execute(tx.as_mut())
        .await?;
    }

    sqlx::query(
        "INSERT INTO book_operation_change (
           operation_id, user_id, book_id, before_revision_number, after_revision_number
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(tx.operation_id().to_uuid())
    .bind(user_id.as_str())
    .bind(book.id().to_uuid())
    .bind(before_revision_number)
    .bind(revision_number)
    .execute(tx.as_mut())
    .await?;

    Ok(revision_number)
}

pub async fn append_book_deletion(
    tx: &mut PgTransaction,
    book_id: uuid::Uuid,
    before_revision_number: i32,
) -> Result<(), DomainError> {
    let user_id = tx.user_id().clone();
    sqlx::query(
        "INSERT INTO book_operation_change (
           operation_id, user_id, book_id, before_revision_number, after_revision_number
         ) VALUES ($1, $2, $3, $4, NULL)",
    )
    .bind(tx.operation_id().to_uuid())
    .bind(user_id.as_str())
    .bind(book_id)
    .bind(before_revision_number)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

pub async fn append_author_revision(
    tx: &mut PgTransaction,
    author: &Author,
    before_revision_number: Option<i32>,
) -> Result<i32, DomainError> {
    let user_id = tx.user_id().clone();
    let latest: Option<i32> = sqlx::query(
        "SELECT MAX(revision_number) AS revision_number
         FROM author_revision
         WHERE author_id = $1 AND user_id = $2",
    )
    .bind(author.id().to_uuid())
    .bind(user_id.as_str())
    .fetch_one(tx.as_mut())
    .await?
    .try_get("revision_number")?;
    let revision_number = latest
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| DomainError::Unexpected("Author revision number overflow".to_owned()))?;

    sqlx::query(
        "INSERT INTO author_revision (
           author_id, revision_number, user_id, name, yomi,
           author_created_at, author_updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(author.id().to_uuid())
    .bind(revision_number)
    .bind(user_id.as_str())
    .bind(author.name().as_str())
    .bind(author.yomi())
    .bind(author.created_at())
    .bind(author.updated_at())
    .execute(tx.as_mut())
    .await?;

    sqlx::query(
        "INSERT INTO author_operation_change (
           operation_id, user_id, author_id, before_revision_number, after_revision_number
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(tx.operation_id().to_uuid())
    .bind(user_id.as_str())
    .bind(author.id().to_uuid())
    .bind(before_revision_number)
    .bind(revision_number)
    .execute(tx.as_mut())
    .await?;

    Ok(revision_number)
}

pub async fn append_author_deletion(
    tx: &mut PgTransaction,
    author_id: uuid::Uuid,
    before_revision_number: i32,
) -> Result<(), DomainError> {
    let user_id = tx.user_id().clone();
    sqlx::query(
        "INSERT INTO author_operation_change (
           operation_id, user_id, author_id, before_revision_number, after_revision_number
         ) VALUES ($1, $2, $3, $4, NULL)",
    )
    .bind(tx.operation_id().to_uuid())
    .bind(user_id.as_str())
    .bind(author_id)
    .bind(before_revision_number)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

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
