#![cfg(feature = "test-with-database")]

use bookshelf_api::{
    domain::{
        entity::{operation::OperationId, user::UserId},
        error::DomainError,
        repository::history_repository::HistoryRepository,
    },
    infrastructure::{history_repository::PgHistoryRepository, undo::PgUndoCommand},
    use_case::port::undo::UndoCommand,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
enum Entity {
    Book,
    Author,
}

impl Entity {
    /// Returns a fixed fixture entity name; no user-controlled identifier is interpolated.
    fn table(self) -> &'static str {
        match self {
            Self::Book => "book",
            Self::Author => "author",
        }
    }
}

/// Captures every current/history table for one tenant or the complete isolated test database.
async fn state(pool: &PgPool, owner: Option<&str>) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut rows = Vec::new();
    for query in [
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM author t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_author t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM operation t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_revision t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM author_revision t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_revision_author t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_operation_change t WHERE $1::text IS NULL OR user_id=$1",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM author_operation_change t WHERE $1::text IS NULL OR user_id=$1",
    ] {
        rows.push(
            sqlx::query_scalar(query)
                .bind(owner)
                .fetch_one(pool)
                .await?,
        );
    }
    Ok(rows)
}

/// Inserts an immutable owned fixture snapshot at the supplied revision.
async fn revision(
    pool: &PgPool,
    entity: Entity,
    id: Uuid,
    owner: &str,
    number: i32,
) -> anyhow::Result<()> {
    let query = match entity {
        Entity::Book => {
            "INSERT INTO book_revision (book_id, user_id, revision_number, title, isbn, read, owned, priority, format, store, book_created_at, book_updated_at) VALUES ($1,$2,$3,'Book','',false,true,4,'Printed','Unknown',current_timestamp,current_timestamp)"
        }
        Entity::Author => {
            "INSERT INTO author_revision (author_id,user_id,revision_number,name,yomi,author_created_at,author_updated_at) VALUES ($1,$2,$3,'Author','',current_timestamp,current_timestamp)"
        }
    };
    sqlx::query(query)
        .bind(id)
        .bind(owner)
        .bind(number)
        .execute(pool)
        .await?;
    Ok(())
}

/// Inserts the current fixture entity without affecting existing history.
async fn current(pool: &PgPool, entity: Entity, id: Uuid, owner: &str) -> anyhow::Result<()> {
    let query = match entity {
        Entity::Book => {
            "INSERT INTO book (id,user_id,title,isbn,read,owned,priority,format,store) VALUES ($1,$2,'Book','',false,true,4,'Printed','Unknown')"
        }
        Entity::Author => "INSERT INTO author (id,user_id,name,yomi) VALUES ($1,$2,'Author','')",
    };
    sqlx::query(query)
        .bind(id)
        .bind(owner)
        .execute(pool)
        .await?;
    Ok(())
}

/// Seeds an owned target and a separate tenant sharing the same entity UUID.
async fn setup(
    pool: &PgPool,
    entity: Entity,
    before: Option<i32>,
    after: Option<i32>,
) -> anyhow::Result<(UserId, OperationId, Uuid)> {
    sqlx::query(
        "INSERT INTO bookshelf_user (id) VALUES ('owner'),('other') ON CONFLICT DO NOTHING",
    )
    .execute(pool)
    .await?;
    let id = Uuid::new_v4();
    let operation = Uuid::new_v4();
    let table = entity.table();
    let kind = match (before, after) {
        (None, _) => "create",
        (_, None) => "delete",
        _ => "update",
    };
    sqlx::query("INSERT INTO operation (id,user_id,type) VALUES ($1,'owner',$2)")
        .bind(operation)
        .bind(format!("{kind}_{table}"))
        .execute(pool)
        .await?;
    for number in 1..=before.into_iter().chain(after).max().unwrap_or(1) {
        revision(pool, entity, id, "owner", number).await?;
    }
    if after.is_some() {
        current(pool, entity, id, "owner").await?;
    }
    sqlx::query(match entity {
        Entity::Book => "INSERT INTO book_operation_change (operation_id,user_id,book_id,before_revision_number,after_revision_number) VALUES ($1,'owner',$2,$3,$4)",
        Entity::Author => "INSERT INTO author_operation_change (operation_id,user_id,author_id,before_revision_number,after_revision_number) VALUES ($1,'owner',$2,$3,$4)",
    })
        .bind(operation).bind(id).bind(before).bind(after).execute(pool).await?;
    // Reusing the UUID in another tenant catches queries that omit owner scope.
    current(pool, entity, id, "other").await?;
    revision(pool, entity, id, "other", 99).await?;
    Ok((UserId::new("owner".into())?, operation.into(), id))
}

/// Checks inverse links, restored persisted values, and tenant independence for both entities.
#[sqlx::test]
async fn eligible_create_update_delete_record_owned_inverse(pool: PgPool) -> anyhow::Result<()> {
    // Separate owners are unnecessary: clear these isolated test fixtures between cases.
    for entity in [Entity::Book, Entity::Author] {
        for (before, after) in [(None, Some(1)), (Some(1), Some(2)), (Some(1), None)] {
            sqlx::query("TRUNCATE bookshelf_user CASCADE")
                .execute(&pool)
                .await?;
            let (owner, target, id) = setup(&pool, entity, before, after).await?;
            if before.is_some() {
                sqlx::query(match entity {
                    Entity::Book => "UPDATE book_revision SET title='Before Book',isbn='9784873115658',read=true,owned=false,priority=99,format='eBook',store='Kindle',purchase_date=DATE '2020-02-29',book_created_at='2001-02-03' WHERE user_id='owner' AND book_id=$1 AND revision_number=1",
                    Entity::Author => "UPDATE author_revision SET name='Before Author',yomi='よみ',author_created_at='2001-02-03' WHERE user_id='owner' AND author_id=$1 AND revision_number=1",
                }).bind(id).execute(&pool).await?;
            }
            let other_before = state(&pool, Some("other")).await?;
            let history = PgHistoryRepository::new(pool.clone());
            assert!(
                history.is_operation_undoable(&owner, &target).await?,
                "{entity:?} {before:?}->{after:?}"
            );
            let inverse = PgUndoCommand::new(pool.clone())
                .undo_operation(&owner, &target)
                .await?;
            let (kind, undo_target): (String, Uuid) = sqlx::query_as(
                "SELECT type,undo_of_operation_id FROM operation WHERE user_id='owner' AND id=$1",
            )
            .bind(inverse.to_uuid())
            .fetch_one(&pool)
            .await?;
            assert_eq!(kind, "undo");
            assert_eq!(undo_target, target.to_uuid());
            let expected_next = before.map(|_| after.unwrap_or(1) + 1);
            let link: (Option<i32>, Option<i32>) = sqlx::query_as(match entity {
                Entity::Book => "SELECT before_revision_number,after_revision_number FROM book_operation_change WHERE user_id='owner' AND operation_id=$1 AND book_id=$2",
                Entity::Author => "SELECT before_revision_number,after_revision_number FROM author_operation_change WHERE user_id='owner' AND operation_id=$1 AND author_id=$2",
            })
                .bind(inverse.to_uuid()).bind(id).fetch_one(&pool).await?;
            assert_eq!(link, (after, expected_next));
            let exists: bool = sqlx::query_scalar(match entity {
                Entity::Book => {
                    "SELECT EXISTS (SELECT 1 FROM book WHERE user_id='owner' AND id=$1)"
                }
                Entity::Author => {
                    "SELECT EXISTS (SELECT 1 FROM author WHERE user_id='owner' AND id=$1)"
                }
            })
            .bind(id)
            .fetch_one(&pool)
            .await?;
            assert_eq!(exists, before.is_some());
            if before.is_some() {
                let matches_source: bool = sqlx::query_scalar(match entity {
                    Entity::Book => "SELECT ROW(b.title,b.isbn,b.read,b.owned,b.priority,b.format,b.store,b.purchase_date,b.created_at) IS NOT DISTINCT FROM ROW(r.title,r.isbn,r.read,r.owned,r.priority,r.format,r.store,r.purchase_date,r.book_created_at) FROM book b JOIN book_revision r ON r.user_id=b.user_id AND r.book_id=b.id AND r.revision_number=1 WHERE b.user_id='owner' AND b.id=$1",
                    Entity::Author => "SELECT ROW(a.name,a.yomi,a.created_at) IS NOT DISTINCT FROM ROW(r.name,r.yomi,r.author_created_at) FROM author a JOIN author_revision r ON r.user_id=a.user_id AND r.author_id=a.id AND r.revision_number=1 WHERE a.user_id='owner' AND a.id=$1",
                }).bind(id).fetch_one(&pool).await?;
                assert!(
                    matches_source,
                    "{entity:?} restored fields must match the before snapshot"
                );
            }

            assert!(!history.is_operation_undoable(&owner, &target).await?);
            assert!(history.is_operation_undoable(&owner, &inverse).await?);
            assert_eq!(state(&pool, Some("other")).await?, other_before);
        }
    }
    Ok(())
}

/// Checks execution revalidation after every existence/revision mismatch invalidates advice.
#[sqlx::test]
async fn stale_advice_is_revalidated_without_writes(pool: PgPool) -> anyhow::Result<()> {
    for entity in [Entity::Book, Entity::Author] {
        for mutation in ["newer revision", "missing current", "recreated current"] {
            sqlx::query("TRUNCATE bookshelf_user CASCADE")
                .execute(&pool)
                .await?;
            let deleted = mutation == "recreated current";
            let (owner, target, id) =
                setup(&pool, entity, Some(1), if deleted { None } else { Some(2) }).await?;
            let history = PgHistoryRepository::new(pool.clone());
            assert!(history.is_operation_undoable(&owner, &target).await?);
            match mutation {
                "newer revision" => revision(&pool, entity, id, "owner", 3).await?,
                "missing current" => {
                    sqlx::query(match entity {
                        Entity::Book => "DELETE FROM book WHERE user_id='owner' AND id=$1",
                        Entity::Author => "DELETE FROM author WHERE user_id='owner' AND id=$1",
                    })
                    .bind(id)
                    .execute(&pool)
                    .await?;
                }
                _ => current(&pool, entity, id, "owner").await?,
            }
            let before = state(&pool, None).await?;
            assert!(!history.is_operation_undoable(&owner, &target).await?);
            assert!(
                matches!(
                    PgUndoCommand::new(pool.clone())
                        .undo_operation(&owner, &target)
                        .await,
                    Err(DomainError::Conflict(_))
                ),
                "{entity:?}: {mutation}"
            );
            assert_eq!(state(&pool, None).await?, before);
        }
    }
    Ok(())
}

/// Checks inaccessible or ineligible targets never produce partial writes.
#[sqlx::test]
async fn missing_foreign_baseline_and_empty_targets_are_rejected(
    pool: PgPool,
) -> anyhow::Result<()> {
    let (owner, target, _) = setup(&pool, Entity::Book, None, Some(1)).await?;
    let other = UserId::new("other".into())?;
    let history = PgHistoryRepository::new(pool.clone());
    let command = PgUndoCommand::new(pool.clone());
    for (user, operation) in [
        (&other, target.clone()),
        (&owner, OperationId::from(Uuid::new_v4())),
    ] {
        let before = state(&pool, None).await?;
        assert!(!history.is_operation_undoable(user, &operation).await?);
        assert!(matches!(
            command.undo_operation(user, &operation).await,
            Err(DomainError::NotFound {
                entity_type: "operation",
                ..
            })
        ));
        assert_eq!(state(&pool, None).await?, before);
    }
    sqlx::query("UPDATE operation SET type='baseline' WHERE user_id='owner' AND id=$1")
        .bind(target.to_uuid())
        .execute(&pool)
        .await?;
    let empty = OperationId::from(Uuid::new_v4());
    sqlx::query("INSERT INTO operation (id,user_id,type) VALUES ($1,'owner','update_book')")
        .bind(empty.to_uuid())
        .execute(&pool)
        .await?;
    for operation in [target, empty] {
        let before = state(&pool, None).await?;
        assert!(!history.is_operation_undoable(&owner, &operation).await?);
        assert!(matches!(
            command.undo_operation(&owner, &operation).await,
            Err(DomainError::Conflict(_))
        ));
        assert_eq!(state(&pool, None).await?, before);
    }
    Ok(())
}

/// Fixes the distinction between advisory revision checks and execution-only constraints.
#[sqlx::test]
async fn eligibility_does_not_guarantee_dependency_or_name_availability(
    pool: PgPool,
) -> anyhow::Result<()> {
    let (owner, target, book) = setup(&pool, Entity::Book, Some(1), Some(2)).await?;
    let missing = Uuid::new_v4();
    sqlx::query("INSERT INTO book_revision_author (user_id,book_id,revision_number,author_id) VALUES ('owner',$1,1,$2)").bind(book).bind(missing).execute(&pool).await?;
    let history = PgHistoryRepository::new(pool.clone());
    assert!(history.is_operation_undoable(&owner, &target).await?);
    let before = state(&pool, None).await?;
    assert!(matches!(
        PgUndoCommand::new(pool.clone())
            .undo_operation(&owner, &target)
            .await,
        Err(DomainError::Conflict(_))
    ));
    assert_eq!(state(&pool, None).await?, before);

    let (owner, target, author) = setup(&pool, Entity::Author, Some(1), Some(2)).await?;
    sqlx::query("UPDATE author SET name='Changed' WHERE user_id='owner' AND id=$1")
        .bind(author)
        .execute(&pool)
        .await?;
    sqlx::query("UPDATE author_revision SET name='Changed' WHERE user_id='owner' AND author_id=$1 AND revision_number=2").bind(author).execute(&pool).await?;
    current(&pool, Entity::Author, Uuid::new_v4(), "owner").await?;
    assert!(history.is_operation_undoable(&owner, &target).await?);
    let before = state(&pool, None).await?;
    assert!(
        matches!(PgUndoCommand::new(pool.clone()).undo_operation(&owner, &target).await, Err(DomainError::AuthorNameConflict(name)) if name == "Author")
    );
    assert_eq!(state(&pool, None).await?, before);
    Ok(())
}

/// Uses observable lock contention to verify one committed inverse and an allowed loser error.
#[sqlx::test]
async fn contending_commands_commit_only_one_inverse(pool: PgPool) -> anyhow::Result<()> {
    let (owner, target, _) = setup(&pool, Entity::Book, Some(1), Some(2)).await?;
    let other_before = state(&pool, Some("other")).await?;
    let application = format!("undo-contract-{}", Uuid::new_v4());
    let options = (*pool.connect_options())
        .clone()
        .application_name(&application);
    let workers = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await?;
    let mut blocker = pool.begin().await?;
    sqlx::query("SELECT id FROM operation WHERE user_id='owner' AND id=$1 FOR UPDATE")
        .bind(target.to_uuid())
        .execute(&mut *blocker)
        .await?;
    let mut tasks = Vec::new();
    for _ in 0..2 {
        let command = PgUndoCommand::new(workers.clone());
        let owner = owner.clone();
        let target = target.clone();
        tasks.push(tokio::spawn(async move {
            command.undo_operation(&owner, &target).await
        }));
    }
    let waiting = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE datname=current_database() AND application_name=$1 AND wait_event_type='Lock' AND query LIKE '%SELECT type FROM operation%'")
                .bind(&application).fetch_one(&pool).await?;
            if count == 2 { return Ok::<_, sqlx::Error>(()); }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }).await;
    // Release the blocker even on timeout, so test failure cannot strand tasks.
    blocker.rollback().await?;
    if !matches!(waiting, Ok(Ok(()))) {
        for task in &tasks {
            task.abort();
        }
        anyhow::bail!("both commands did not reach the operation lock: {waiting:?}");
    }
    let results = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let mut results = Vec::new();
        for task in &mut tasks {
            results.push(task.await?);
        }
        Ok::<_, tokio::task::JoinError>(results)
    })
    .await;
    if results.is_err() {
        for task in &tasks {
            task.abort();
        }
    }
    let results = results??;
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    for result in results {
        match result {
            Ok(_) | Err(DomainError::Conflict(_)) => (),
            Err(DomainError::InfrastructureError(error)) => {
                let sqlx::Error::Database(error) =
                    error.downcast_ref::<sqlx::Error>().expect("SQLx failure")
                else {
                    panic!("unexpected error: {error}")
                };
                assert_eq!(error.code().as_deref(), Some("40001"));
            }
            Err(error) => panic!("unexpected loser error: {error}"),
        }
    }
    let committed: i64 = sqlx::query_scalar("SELECT count(*) FROM operation WHERE user_id='owner' AND type='undo' AND undo_of_operation_id=$1").bind(target.to_uuid()).fetch_one(&pool).await?;
    assert_eq!(committed, 1);
    let revisions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM book_revision WHERE user_id='owner'")
            .fetch_one(&pool)
            .await?;
    assert_eq!(revisions, 3);
    let links: i64 = sqlx::query_scalar("SELECT count(*) FROM book_operation_change WHERE user_id='owner' AND before_revision_number=2 AND after_revision_number=3").fetch_one(&pool).await?;
    assert_eq!(links, 1);
    assert_eq!(state(&pool, Some("other")).await?, other_before);
    workers.close().await;
    Ok(())
}
