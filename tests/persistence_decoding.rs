#![cfg(feature = "test-with-database")]

use bookshelf_api::{
    domain::{
        entity::{book::BookId, operation::NewOperation, revision::RevisionNumber, user::UserId},
        error::DomainError,
        repository::{
            book_repository::BookRepository, history_repository::HistoryRepository,
            transaction::TransactionManager,
        },
    },
    infrastructure::{
        backup::PgBackupQuery, book_repository::PgBookRepository,
        history_repository::PgHistoryRepository, transaction::PgTransactionManager,
    },
    use_case::port::backup::BackupQueryPort,
};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn corrupt_enum_is_internal_in_current_history_and_backup_reads(
    pool: PgPool,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO bookshelf_user (id) VALUES ('decode-owner')")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT INTO book_format (format) VALUES ('private-format')")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT INTO book_store (store) VALUES ('private-store')")
        .execute(&pool)
        .await?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO book (id, user_id, title, isbn, read, owned, priority, format, store) VALUES ($1, 'decode-owner', 'Valid', '', false, true, 50, 'Printed', 'Unknown')")
        .bind(id).execute(&pool).await?;
    sqlx::query("INSERT INTO book_revision (book_id, revision_number, user_id, title, isbn, read, owned, priority, format, store, book_created_at, book_updated_at) VALUES ($1, 1, 'decode-owner', 'Valid', '', false, true, 50, 'Printed', 'Unknown', current_timestamp, current_timestamp)")
        .bind(id).execute(&pool).await?;
    let owner = UserId::new("decode-owner".into())?;
    let book = BookId::new(id)?;
    let repository = PgBookRepository::new(pool.clone());
    let history = PgHistoryRepository::new(pool.clone());
    let backup = PgBackupQuery::new(pool.clone());
    for (format, store) in [("private-format", "Unknown"), ("Printed", "private-store")] {
        sqlx::query("UPDATE book SET format=$1, store=$2 WHERE id=$3")
            .bind(format)
            .bind(store)
            .bind(id)
            .execute(&pool)
            .await?;
        sqlx::query("UPDATE book_revision SET format=$1, store=$2 WHERE book_id=$3")
            .bind(format)
            .bind(store)
            .bind(id)
            .execute(&pool)
            .await?;
        assert!(matches!(
            repository.find_by_id(&owner, &book).await,
            Err(DomainError::Unexpected(_))
        ));
        assert!(matches!(
            history
                .find_book_revision(&owner, &book, RevisionNumber::FIRST)
                .await,
            Err(DomainError::Unexpected(_))
        ));
        assert!(matches!(
            backup.snapshot(&owner).await,
            Err(DomainError::Unexpected(_))
        ));
        // Keep current data valid to prove full export reaches its historical decoder.
        sqlx::query("UPDATE book SET format='Printed', store='Unknown' WHERE id=$1")
            .bind(id)
            .execute(&pool)
            .await?;
        assert!(backup.snapshot(&owner).await.is_ok());
        assert!(matches!(
            backup.full(&owner).await,
            Err(DomainError::Unexpected(_))
        ));
        // Restore must reject the corrupted source without changing current state or history.
        let before = stored_state(&pool).await?;
        let manager = PgTransactionManager::new(pool.clone());
        let mut tx = manager
            .begin_operation(&owner, &NewOperation::restore_book(1))
            .await?;
        assert!(matches!(
            repository.restore_revision(&mut tx, &book, 1).await,
            Err(DomainError::Unexpected(_))
        ));
        manager.rollback(tx).await?;
        assert_eq!(stored_state(&pool).await?, before);
    }
    Ok(())
}

async fn stored_state(pool: &PgPool) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut result = Vec::new();
    for query in [
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book t WHERE user_id='decode-owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_revision t WHERE user_id='decode-owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM operation t WHERE user_id='decode-owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_operation_change t WHERE user_id='decode-owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_author t WHERE user_id='decode-owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_revision_author t WHERE user_id='decode-owner'",
    ] {
        result.push(sqlx::query_scalar(query).fetch_one(pool).await?);
    }
    Ok(result)
}
