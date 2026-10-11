#![cfg(feature = "test-with-database")]

use bookshelf_api::{
    domain::{
        entity::{
            author::AuthorId,
            operation::{NewOperation, OperationId},
            user::UserId,
        },
        error::DomainError,
        repository::{
            author_repository::AuthorRepository, history_repository::HistoryRepository,
            transaction::TransactionManager,
        },
    },
    infrastructure::{
        author_repository::PgAuthorRepository, history_repository::PgHistoryRepository,
        transaction::PgTransactionManager,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

async fn state(pool: &PgPool) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut result = Vec::new();
    for query in [
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM author t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_author t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM operation t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM author_revision t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_revision t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_revision_author t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM author_operation_change t WHERE user_id = 'owner'",
        "SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM book_operation_change t WHERE user_id = 'owner'",
    ] {
        result.push(sqlx::query_scalar(query).fetch_one(pool).await?);
    }
    Ok(result)
}

async fn setup(pool: &PgPool, multiple: bool) -> anyhow::Result<(OperationId, Vec<Uuid>)> {
    sqlx::query("INSERT INTO bookshelf_user (id) VALUES ('owner'), ('other')")
        .execute(pool)
        .await?;
    let operation = Uuid::new_v4();
    sqlx::query("INSERT INTO operation (id, user_id, type) VALUES ($1, 'owner', 'update_author')")
        .bind(operation)
        .execute(pool)
        .await?;
    // Deterministic IDs ensure the second inverse hits the collision after the first has written.
    let ids = if multiple {
        vec![Uuid::from_u128(1), Uuid::from_u128(2)]
    } else {
        vec![Uuid::from_u128(2)]
    };
    for id in &ids {
        let old = format!("Original-{id}");
        let new = format!("Changed-{id}");
        sqlx::query("INSERT INTO author (id, user_id, name, yomi) VALUES ($1, 'owner', $2, '')")
            .bind(id)
            .bind(&new)
            .execute(pool)
            .await?;
        for (revision, name) in [(1, &old), (2, &new)] {
            sqlx::query("INSERT INTO author_revision (author_id, revision_number, user_id, name, yomi, author_created_at, author_updated_at) VALUES ($1, $2, 'owner', $3, '', current_timestamp, current_timestamp)")
                .bind(id).bind(revision).bind(name).execute(pool).await?;
        }
        sqlx::query("INSERT INTO author_operation_change (operation_id, user_id, author_id, before_revision_number, after_revision_number) VALUES ($1, 'owner', $2, 1, 2)")
            .bind(operation).bind(id).execute(pool).await?;
    }
    let taken = format!("Original-{}", ids.last().unwrap());
    sqlx::query("INSERT INTO author (id, user_id, name, yomi) VALUES ($1, 'owner', $2, '')")
        .bind(Uuid::from_u128(3))
        .bind(&taken)
        .execute(pool)
        .await?;
    // The same name in another tenant must remain legal.
    sqlx::query("INSERT INTO author (id, user_id, name, yomi) VALUES ($1, 'other', $2, '')")
        .bind(Uuid::from_u128(4))
        .bind(&taken)
        .execute(pool)
        .await?;
    Ok((OperationId::from(operation), ids))
}

#[sqlx::test]
async fn restore_name_conflict_preserves_all_rows_and_recovers(pool: PgPool) -> anyhow::Result<()> {
    let (_, ids) = setup(&pool, false).await?;
    let owner = UserId::new("owner".into())?;
    let authors = PgAuthorRepository::new(pool.clone());
    let manager = PgTransactionManager::new(pool.clone());
    let before = state(&pool).await?;
    let mut tx = manager
        .begin_operation(&owner, &NewOperation::restore_author(1))
        .await?;
    let result = authors
        .restore_revision(&mut tx, &AuthorId::from(ids[0]), 1)
        .await;
    assert!(
        matches!(result, Err(DomainError::AuthorNameConflict(name)) if name == format!("Original-{}", ids[0]))
    );
    manager.rollback(tx).await?;
    assert_eq!(state(&pool).await?, before);
    sqlx::query("DELETE FROM author WHERE user_id='owner' AND id=$1")
        .bind(Uuid::from_u128(3))
        .execute(&pool)
        .await?;
    let mut tx = manager
        .begin_operation(&owner, &NewOperation::restore_author(1))
        .await?;
    let restored = authors
        .restore_revision(&mut tx, &AuthorId::from(ids[0]), 1)
        .await?;
    assert_eq!(restored.revision_number.value(), 3);
    manager.commit(tx).await?;
    Ok(())
}

#[sqlx::test]
async fn undo_later_name_conflict_rolls_back_earlier_inverse_and_history(
    pool: PgPool,
) -> anyhow::Result<()> {
    let (operation, ids) = setup(&pool, true).await?;
    let owner = UserId::new("owner".into())?;
    let history = PgHistoryRepository::new(pool.clone());
    let before = state(&pool).await?;
    let result = history.undo_operation(&owner, &operation).await;
    assert!(
        matches!(result, Err(DomainError::AuthorNameConflict(name)) if name == format!("Original-{}", ids[1]))
    );
    // An ordinary query waits for any dropped-transaction rollback to finish in the pool.
    assert_eq!(state(&pool).await?, before);
    sqlx::query("DELETE FROM author WHERE user_id='owner' AND id=$1")
        .bind(Uuid::from_u128(3))
        .execute(&pool)
        .await?;
    history.undo_operation(&owner, &operation).await?;
    for id in ids {
        let (name, revision): (String, i32) = sqlx::query_as("SELECT name, (SELECT MAX(revision_number) FROM author_revision WHERE user_id='owner' AND author_id=$1) FROM author WHERE user_id='owner' AND id=$1")
            .bind(id).fetch_one(&pool).await?;
        assert_eq!(name, format!("Original-{id}"));
        assert_eq!(revision, 3);
    }
    Ok(())
}

#[sqlx::test]
async fn restore_classifies_write_collision_after_precheck(pool: PgPool) -> anyhow::Result<()> {
    let (_, ids) = setup(&pool, false).await?;
    sqlx::query("DELETE FROM author WHERE user_id='owner' AND id=$1")
        .bind(Uuid::from_u128(3))
        .execute(&pool)
        .await?;
    sqlx::raw_sql("CREATE FUNCTION inject_author_collision() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.user_id = 'owner' AND NEW.id = '00000000-0000-0000-0000-000000000002' THEN INSERT INTO author (id, user_id, name, yomi) VALUES ('00000000-0000-0000-0000-000000000003', NEW.user_id, NEW.name, ''); END IF; RETURN NEW; END $$; CREATE TRIGGER inject_author_collision BEFORE INSERT ON author FOR EACH ROW EXECUTE FUNCTION inject_author_collision();").execute(&pool).await?;
    let before = state(&pool).await?;
    let manager = PgTransactionManager::new(pool.clone());
    let owner = UserId::new("owner".into())?;
    let mut tx = manager
        .begin_operation(&owner, &NewOperation::restore_author(1))
        .await?;
    let result = PgAuthorRepository::new(pool.clone())
        .restore_revision(&mut tx, &AuthorId::from(ids[0]), 1)
        .await;
    assert!(matches!(result, Err(DomainError::AuthorNameConflict(_))));
    manager.rollback(tx).await?;
    assert_eq!(state(&pool).await?, before);
    Ok(())
}

#[sqlx::test]
async fn undo_corrupt_author_name_rolls_back_earlier_inverse_and_history(
    pool: PgPool,
) -> anyhow::Result<()> {
    let (operation, ids) = setup(&pool, true).await?;
    sqlx::query("UPDATE author_revision SET name='' WHERE user_id='owner' AND author_id=$1 AND revision_number=1")
        .bind(ids[1]).execute(&pool).await?;
    let owner = UserId::new("owner".into())?;
    let history = PgHistoryRepository::new(pool.clone());
    let before = state(&pool).await?;
    assert!(matches!(
        history.undo_operation(&owner, &operation).await,
        Err(DomainError::Unexpected(_))
    ));
    assert_eq!(state(&pool).await?, before);
    sqlx::query("UPDATE author_revision SET name='Recovered' WHERE user_id='owner' AND author_id=$1 AND revision_number=1")
        .bind(ids[1]).execute(&pool).await?;
    history.undo_operation(&owner, &operation).await?;
    let name: String =
        sqlx::query_scalar("SELECT name FROM author WHERE user_id='owner' AND id=$1")
            .bind(ids[1])
            .fetch_one(&pool)
            .await?;
    assert_eq!(name, "Recovered");
    Ok(())
}
