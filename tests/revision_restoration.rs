#![cfg(feature = "test-with-database")]

use bookshelf_api::{
    domain::{
        entity::{
            author::AuthorId,
            book::BookId,
            operation::{NewOperation, OperationId},
            user::UserId,
        },
        repository::{
            author_repository::AuthorRepository, book_repository::BookRepository,
            transaction::TransactionManager,
        },
    },
    infrastructure::{
        author_repository::PgAuthorRepository, book_repository::PgBookRepository,
        transaction::PgTransactionManager, undo::PgUndoCommand,
    },
    use_case::port::undo::UndoCommand,
};
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// Seeds divergent current/source values and a second tenant sharing both UUIDs.
async fn fixture(
    pool: &PgPool,
    deleted: bool,
    empty: bool,
) -> anyhow::Result<(UserId, OperationId, Uuid, Uuid)> {
    sqlx::query("INSERT INTO bookshelf_user (id) VALUES ('owner'),('other')")
        .execute(pool)
        .await?;
    let book = Uuid::new_v4();
    let author = Uuid::new_v4();
    let target = Uuid::new_v4();
    sqlx::query("INSERT INTO operation (id,user_id,type) VALUES ($1,'owner',$2)")
        .bind(target)
        .bind(if deleted {
            "delete_book"
        } else {
            "update_book"
        })
        .execute(pool)
        .await?;
    for user in ["owner", "other"] {
        sqlx::query("INSERT INTO author (id,user_id,name,yomi,created_at,updated_at) VALUES ($1,$2,'Changed Author','changed','2005-01-02','2006-03-04')").bind(author).bind(user).execute(pool).await?;
        sqlx::query("INSERT INTO book (id,user_id,title,isbn,read,owned,priority,format,store,purchase_date,created_at,updated_at) VALUES ($1,$2,'Changed Book','',false,true,4,'Printed','Unknown','2025-01-01','2005-01-02','2006-03-04')").bind(book).bind(user).execute(pool).await?;
        sqlx::query("INSERT INTO book_author (user_id,book_id,author_id) VALUES ($1,$2,$3)")
            .bind(user)
            .bind(book)
            .bind(author)
            .execute(pool)
            .await?;
        // Owner and other tenant have different source values under the same UUIDs.
        sqlx::query("INSERT INTO author_revision (author_id,user_id,revision_number,name,yomi,author_created_at,author_updated_at) VALUES ($1,$2,1,$3,'よみ','2001-02-03 04:05:06.123456+00','2002-03-04')")
            .bind(author).bind(user).bind(if user == "owner" {"Original Author"} else {"Other Author"}).execute(pool).await?;
        sqlx::query("INSERT INTO book_revision (book_id,user_id,revision_number,title,isbn,read,owned,priority,format,store,purchase_date,book_created_at,book_updated_at) VALUES ($1,$2,1,$3,'9784873115658',true,false,99,'eBook','Kindle',CASE WHEN $4 THEN NULL ELSE DATE '2020-02-29' END,'2001-02-03 04:05:06.123456+00','2002-03-04')")
            .bind(book).bind(user).bind(if user == "owner" {"Original Book"} else {"Other Book"}).bind(empty).execute(pool).await?;
        if !empty {
            sqlx::query("INSERT INTO book_revision_author (user_id,book_id,revision_number,author_id) VALUES ($1,$2,1,$3)").bind(user).bind(book).bind(author).execute(pool).await?;
        }
        // The other tenant has a higher revision; allocation must remain owner-scoped.
        let latest = if user == "owner" { 2 } else { 99 };
        sqlx::query("INSERT INTO author_revision (author_id,user_id,revision_number,name,yomi,author_created_at,author_updated_at) SELECT id,user_id,$3,name,yomi,created_at,updated_at FROM author WHERE id=$1 AND user_id=$2").bind(author).bind(user).bind(latest).execute(pool).await?;
        sqlx::query("INSERT INTO book_revision (book_id,user_id,revision_number,title,isbn,read,owned,priority,format,store,purchase_date,book_created_at,book_updated_at) SELECT id,user_id,$3,title,isbn,read,owned,priority,format,store,purchase_date,created_at,updated_at FROM book WHERE id=$1 AND user_id=$2").bind(book).bind(user).bind(latest).execute(pool).await?;
        sqlx::query("INSERT INTO book_revision_author (user_id,book_id,revision_number,author_id) VALUES ($1,$2,$3,$4)").bind(user).bind(book).bind(latest).bind(author).execute(pool).await?;
    }
    let after = if deleted { None } else { Some(2) };
    sqlx::query("INSERT INTO book_operation_change (operation_id,user_id,book_id,before_revision_number,after_revision_number) VALUES ($1,'owner',$2,1,$3)").bind(target).bind(book).bind(after).execute(pool).await?;
    sqlx::query("INSERT INTO author_operation_change (operation_id,user_id,author_id,before_revision_number,after_revision_number) VALUES ($1,'owner',$2,1,$3)").bind(target).bind(author).bind(after).execute(pool).await?;
    if deleted {
        sqlx::query("DELETE FROM book_author WHERE user_id='owner'")
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM book WHERE user_id='owner'")
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM author WHERE user_id='owner'")
            .execute(pool)
            .await?;
    }
    Ok((UserId::new("owner".into())?, target.into(), book, author))
}

/// Removes history metadata and aligns lifecycle field names for full payload comparisons.
fn payload(mut value: Value, entity: &str, historical: bool) -> Value {
    let row = value.as_object_mut().unwrap();
    row.remove("user_id");
    if historical {
        row.remove(&format!("{entity}_id"));
        row.remove("revision_number");
        row.remove("created_at");
        let created = row.remove(&format!("{entity}_created_at")).unwrap();
        let updated = row.remove(&format!("{entity}_updated_at")).unwrap();
        row.insert("created_at".into(), created);
        row.insert("updated_at".into(), updated);
    } else {
        row.remove("id");
    }
    value
}

/// Loads owned source payloads before restoration so expected values cannot drift.
async fn source(pool: &PgPool) -> anyhow::Result<Vec<Value>> {
    let mut values = Vec::new();
    for (entity, query) in [
        (
            "book",
            "SELECT to_jsonb(t) FROM book_revision t WHERE user_id='owner' AND revision_number=1",
        ),
        (
            "author",
            "SELECT to_jsonb(t) FROM author_revision t WHERE user_id='owner' AND revision_number=1",
        ),
    ] {
        values.push(payload(
            sqlx::query_scalar(query).fetch_one(pool).await?,
            entity,
            true,
        ));
    }
    Ok(values)
}

/// Captures the other tenant current data, history, and author links.
async fn other_state(pool: &PgPool) -> anyhow::Result<Vec<Value>> {
    let mut values = Vec::new();
    for query in [
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text) FROM book t WHERE user_id='other'",
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text) FROM author t WHERE user_id='other'",
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text) FROM book_author t WHERE user_id='other'",
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text) FROM book_revision t WHERE user_id='other'",
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text) FROM author_revision t WHERE user_id='other'",
        "SELECT jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text) FROM book_revision_author t WHERE user_id='other'",
    ] {
        values.push(sqlx::query_scalar(query).fetch_one(pool).await?);
    }
    Ok(values)
}

/// Compares every restored payload and relationship against its source and fresh revision.
async fn assert_restored(pool: &PgPool, originals: Vec<Value>, empty: bool) -> anyhow::Result<()> {
    for ((entity, current_query, history_query), mut original) in [
        (
            "book",
            "SELECT to_jsonb(t) FROM book t WHERE user_id='owner'",
            "SELECT to_jsonb(t) FROM book_revision t WHERE user_id='owner' AND revision_number=3",
        ),
        (
            "author",
            "SELECT to_jsonb(t) FROM author t WHERE user_id='owner'",
            "SELECT to_jsonb(t) FROM author_revision t WHERE user_id='owner' AND revision_number=3",
        ),
    ]
    .into_iter()
    .zip(originals)
    {
        let mut current = payload(
            sqlx::query_scalar(current_query).fetch_one(pool).await?,
            entity,
            false,
        );
        let history = payload(
            sqlx::query_scalar(history_query).fetch_one(pool).await?,
            entity,
            true,
        );
        assert_eq!(
            current, history,
            "persisted {entity} and new revision differ"
        );
        // The source lifecycle start survives, while the update time becomes a fresh value.
        assert_ne!(current["updated_at"], original["updated_at"]);
        current.as_object_mut().unwrap().remove("updated_at");
        original.as_object_mut().unwrap().remove("updated_at");
        assert_eq!(
            current, original,
            "restored {entity} fields differ from source"
        );
    }
    let current: Vec<Uuid> = sqlx::query_scalar(
        "SELECT author_id FROM book_author WHERE user_id='owner' ORDER BY author_id",
    )
    .fetch_all(pool)
    .await?;
    let history: Vec<Uuid> = sqlx::query_scalar("SELECT author_id FROM book_revision_author WHERE user_id='owner' AND revision_number=3 ORDER BY author_id").fetch_all(pool).await?;
    let original: Vec<Uuid> = sqlx::query_scalar("SELECT author_id FROM book_revision_author WHERE user_id='owner' AND revision_number=1 ORDER BY author_id").fetch_all(pool).await?;
    assert_eq!(current, original);
    assert_eq!(history, original);
    assert_eq!(current.is_empty(), empty);
    Ok(())
}

/// Verifies ordinary restore, including resurrection, empty links, dates, and returned lifecycle times.
#[sqlx::test]
async fn ordinary_restore_preserves_all_fields_and_returned_timestamps(
    pool: PgPool,
) -> anyhow::Result<()> {
    for (deleted, empty) in [(false, false), (false, true), (true, false), (true, true)] {
        sqlx::query("TRUNCATE bookshelf_user CASCADE")
            .execute(&pool)
            .await?;
        let (owner, _, book, author) = fixture(&pool, deleted, empty).await?;
        let originals = source(&pool).await?;
        let other = other_state(&pool).await?;
        let manager = PgTransactionManager::new(pool.clone());
        let mut tx = manager
            .begin_operation(&owner, &NewOperation::restore_author(1))
            .await?;
        let author_operation = tx.operation_id();
        let restored_author = PgAuthorRepository::new(pool.clone())
            .restore_revision(&mut tx, &AuthorId::from(author), 1)
            .await?;
        assert_eq!(restored_author.revision_number.value(), 3);
        manager.commit(tx).await?;
        let mut tx = manager
            .begin_operation(&owner, &NewOperation::restore_book(1))
            .await?;
        let book_operation = tx.operation_id();
        let restored_book = PgBookRepository::new(pool.clone())
            .restore_revision(&mut tx, &BookId::new(book)?, 1)
            .await?;
        assert_eq!(restored_book.revision_number.value(), 3);
        manager.commit(tx).await?;
        let (created, updated): (OffsetDateTime, OffsetDateTime) =
            sqlx::query_as("SELECT created_at,updated_at FROM book WHERE user_id='owner'")
                .fetch_one(&pool)
                .await?;
        assert_eq!(restored_book.entity.created_at(), &created);
        assert_eq!(restored_book.entity.updated_at(), &updated);
        let (created, updated): (OffsetDateTime, OffsetDateTime) =
            sqlx::query_as("SELECT created_at,updated_at FROM author WHERE user_id='owner'")
                .fetch_one(&pool)
                .await?;
        assert_eq!(restored_author.entity.created_at(), &created);
        assert_eq!(restored_author.entity.updated_at(), &updated);
        for (operation, query) in [
            (
                book_operation,
                "SELECT before_revision_number,after_revision_number FROM book_operation_change WHERE user_id='owner' AND operation_id=$1",
            ),
            (
                author_operation,
                "SELECT before_revision_number,after_revision_number FROM author_operation_change WHERE user_id='owner' AND operation_id=$1",
            ),
        ] {
            let link: (Option<i32>, Option<i32>) = sqlx::query_as(query)
                .bind(operation.to_uuid())
                .fetch_one(&pool)
                .await?;
            assert_eq!(link, (if deleted { None } else { Some(2) }, Some(3)));
        }
        assert_restored(&pool, originals, empty).await?;
        assert_eq!(other_state(&pool).await?, other);
    }
    Ok(())
}

/// Verifies atomic multi-entity Undo retains source values and transaction timestamp semantics.
#[sqlx::test]
async fn undo_restore_preserves_all_fields_and_transaction_timestamps(
    pool: PgPool,
) -> anyhow::Result<()> {
    for (deleted, empty) in [(false, false), (false, true), (true, false), (true, true)] {
        sqlx::query("TRUNCATE bookshelf_user CASCADE")
            .execute(&pool)
            .await?;
        let (owner, target, _, _) = fixture(&pool, deleted, empty).await?;
        let originals = source(&pool).await?;
        let other = other_state(&pool).await?;
        let inverse = PgUndoCommand::new(pool.clone())
            .undo_operation(&owner, &target)
            .await?;
        let operation_time: OffsetDateTime =
            sqlx::query_scalar("SELECT created_at FROM operation WHERE user_id='owner' AND id=$1")
                .bind(inverse.to_uuid())
                .fetch_one(&pool)
                .await?;
        for query in [
            "SELECT updated_at FROM book WHERE user_id='owner'",
            "SELECT updated_at FROM author WHERE user_id='owner'",
        ] {
            let updated: OffsetDateTime = sqlx::query_scalar(query).fetch_one(&pool).await?;
            assert_eq!(updated, operation_time);
        }
        for query in [
            "SELECT before_revision_number,after_revision_number FROM book_operation_change WHERE user_id='owner' AND operation_id=$1",
            "SELECT before_revision_number,after_revision_number FROM author_operation_change WHERE user_id='owner' AND operation_id=$1",
        ] {
            let link: (Option<i32>, Option<i32>) = sqlx::query_as(query)
                .bind(inverse.to_uuid())
                .fetch_one(&pool)
                .await?;
            assert_eq!(link, (if deleted { None } else { Some(2) }, Some(3)));
        }
        assert_restored(&pool, originals, empty).await?;
        assert_eq!(other_state(&pool).await?, other);
    }
    Ok(())
}

/// Captures all current/history rows to detect even partial inverse writes on failure.
async fn all_state(pool: &PgPool) -> anyhow::Result<Vec<Value>> {
    let mut rows = Vec::new();
    for query in [
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM book t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM author t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM book_author t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM operation t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM book_revision t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM author_revision t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM book_revision_author t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM book_operation_change t",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text),'[]'::jsonb) FROM author_operation_change t",
    ] {
        rows.push(sqlx::query_scalar(query).fetch_one(pool).await?);
    }
    Ok(rows)
}

/// Proves saturated history rejects ordinary restore and rolls back an earlier author inverse.
#[sqlx::test]
async fn revision_overflow_rolls_back_restore_and_earlier_undo_inverse(
    pool: PgPool,
) -> anyhow::Result<()> {
    use bookshelf_api::domain::error::DomainError;
    let (owner, target, book, _) = fixture(&pool, false, false).await?;
    // Build a valid saturated history, keeping foreign keys valid at each step.
    sqlx::query(
        "UPDATE book_operation_change SET after_revision_number=NULL WHERE user_id='owner'",
    )
    .execute(&pool)
    .await?;
    sqlx::query("DELETE FROM book_revision_author WHERE user_id='owner' AND revision_number=2")
        .execute(&pool)
        .await?;
    sqlx::query(
        "UPDATE book_revision SET revision_number=$1 WHERE user_id='owner' AND revision_number=2",
    )
    .bind(i32::MAX)
    .execute(&pool)
    .await?;
    sqlx::query("INSERT INTO book_revision_author (user_id,book_id,revision_number,author_id) SELECT user_id,book_id,$1,author_id FROM book_author WHERE user_id='owner'").bind(i32::MAX).execute(&pool).await?;
    sqlx::query("UPDATE book_operation_change SET after_revision_number=$1 WHERE user_id='owner'")
        .bind(i32::MAX)
        .execute(&pool)
        .await?;
    let before = all_state(&pool).await?;
    let manager = PgTransactionManager::new(pool.clone());
    let mut tx = manager
        .begin_operation(&owner, &NewOperation::restore_book(1))
        .await?;
    assert!(
        matches!(PgBookRepository::new(pool.clone()).restore_revision(&mut tx, &BookId::new(book)?, 1).await, Err(DomainError::Unexpected(message)) if message == "Book revision number overflow")
    );
    manager.rollback(tx).await?;
    assert_eq!(all_state(&pool).await?, before);
    // The author inverse is written before the saturated book is reached.
    assert!(
        matches!(PgUndoCommand::new(pool.clone()).undo_operation(&owner, &target).await, Err(DomainError::Unexpected(message)) if message == "Book revision number overflow")
    );
    assert_eq!(all_state(&pool).await?, before);
    Ok(())
}
