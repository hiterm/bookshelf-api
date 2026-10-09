#![cfg(feature = "test-with-database")]
use bookshelf_api::domain::repository::history_repository::HistoryRepository;
use bookshelf_api::{
    common::types::{BookFormat, BookStore},
    domain::entity::{operation::OperationId, user::UserId},
    infrastructure::{
        author_repository::PgAuthorRepository, book_repository::PgBookRepository,
        history_repository::PgHistoryRepository, transaction::PgTransactionManager,
    },
    use_case::{
        dto::book::{CreateBookDto, UpdateBookDto},
        error::UseCaseError,
        interactor::book::BookCommandInteractor,
        traits::book::BookCommandUseCase,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

fn command(
    pool: &PgPool,
) -> BookCommandInteractor<PgBookRepository, PgAuthorRepository, PgTransactionManager> {
    BookCommandInteractor::new(
        PgBookRepository::new(pool.clone()),
        PgAuthorRepository::new(pool.clone()),
        PgTransactionManager::new(pool.clone()),
    )
}
fn input(names: &[&str]) -> CreateBookDto {
    let mut dto = CreateBookDto::new(
        "Atomic".into(),
        vec![],
        "".into(),
        false,
        false,
        50,
        BookFormat::Unknown,
        BookStore::Unknown,
    );
    dto.new_author_names = names.iter().map(|s| s.to_string()).collect();
    dto
}
fn update(id: &str, names: &[&str]) -> UpdateBookDto {
    let mut dto = UpdateBookDto::new(
        id.into(),
        "Updated".into(),
        vec![],
        "".into(),
        false,
        false,
        50,
        BookFormat::Unknown,
        BookStore::Unknown,
    );
    dto.new_author_names = names.iter().map(|s| s.to_string()).collect();
    dto
}
async fn user(pool: &PgPool, id: &str) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO bookshelf_user (id) VALUES ($1)")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
async fn counts(pool: &PgPool) -> anyhow::Result<(i64, i64, i64, i64, i64, i64)> {
    Ok(sqlx::query_as("SELECT (SELECT count(*) FROM author), (SELECT count(*) FROM book), (SELECT count(*) FROM operation), (SELECT count(*) FROM author_revision), (SELECT count(*) FROM book_revision), (SELECT count(*) FROM book_author)").fetch_one(pool).await?)
}
#[sqlx::test]
async fn create_update_history_and_undo(pool: PgPool) -> anyhow::Result<()> {
    user(&pool, "u").await?;
    let cmd = command(&pool);
    let created = cmd.create("u", input(&["B", "A", "A"])).await?;
    assert_eq!(created.value.author_ids.len(), 2);
    let changes: i64 =
        sqlx::query_scalar("SELECT count(*) FROM author_operation_change WHERE operation_id = $1")
            .bind(Uuid::parse_str(&created.operation_id)?)
            .fetch_one(&pool)
            .await?;
    assert_eq!(changes, 2);
    let before = counts(&pool).await?;
    let mut dto = update(&created.value.id, &["C"]);
    dto.author_ids = vec![created.value.author_ids[0].clone(); 2];
    let edited = cmd.update("u", dto).await?;
    assert_eq!(edited.value.author_ids.len(), 2);
    assert_eq!(edited.revision_number, 2);
    PgHistoryRepository::new(pool.clone())
        .undo_operation(
            &UserId::new("u".into())?,
            &OperationId::try_from(edited.operation_id.as_str()).map_err(anyhow::Error::msg)?,
        )
        .await?;
    let after = counts(&pool).await?;
    assert_eq!(after.0, before.0);
    assert_eq!(after.1, before.1);
    let title: String = sqlx::query_scalar("SELECT title FROM book WHERE id=$1")
        .bind(Uuid::parse_str(&created.value.id)?)
        .fetch_one(&pool)
        .await?;
    assert_eq!(title, "Atomic");
    Ok(())
}
#[sqlx::test]
async fn conflict_rolls_back_earlier_author_and_all_history_for_create_and_update(
    pool: PgPool,
) -> anyhow::Result<()> {
    user(&pool, "u").await?;
    let cmd = command(&pool);
    let original = cmd.create("u", input(&["Z"])).await?;
    let before = counts(&pool).await?;
    for result in [
        cmd.create("u", input(&["A", "Z"])).await,
        cmd.update("u", update(&original.value.id, &["A", "Z"]))
            .await,
    ] {
        assert!(matches!(result,Err(UseCaseError::AuthorNameConflict(name)) if name=="Z"));
        assert_eq!(counts(&pool).await?, before);
    }
    Ok(())
}
#[sqlx::test]
async fn book_write_failure_rolls_back_new_authors_for_create_and_update(
    pool: PgPool,
) -> anyhow::Result<()> {
    user(&pool, "u").await?;
    let cmd = command(&pool);
    let original = cmd.create("u", input(&[])).await?;
    sqlx::raw_sql("CREATE FUNCTION reject_atomic_book() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected book failure'; END $$; CREATE TRIGGER reject_atomic_book BEFORE INSERT OR UPDATE ON book FOR EACH ROW EXECUTE FUNCTION reject_atomic_book();").execute(&pool).await?;
    let before = counts(&pool).await?;
    assert!(cmd.create("u", input(&["A"])).await.is_err());
    assert_eq!(counts(&pool).await?, before);
    assert!(
        cmd.update("u", update(&original.value.id, &["A"]))
            .await
            .is_err()
    );
    assert_eq!(counts(&pool).await?, before);
    Ok(())
}
#[sqlx::test]
async fn rejects_invalid_or_foreign_references_and_allows_other_tenant_names(
    pool: PgPool,
) -> anyhow::Result<()> {
    user(&pool, "u").await?;
    user(&pool, "v").await?;
    let cmd = command(&pool);
    let foreign = cmd.create("v", input(&["A"])).await?;
    let before = counts(&pool).await?;
    for id in [
        foreign.value.author_ids[0].clone(),
        Uuid::new_v4().to_string(),
        "invalid".into(),
    ] {
        let mut dto = input(&["B"]);
        dto.author_ids = vec![id];
        assert!(cmd.create("u", dto).await.is_err());
        assert_eq!(counts(&pool).await?, before);
    }
    assert!(cmd.create("u", input(&[""])).await.is_err());
    assert_eq!(counts(&pool).await?, before);
    cmd.create("u", input(&["A"])).await?;
    Ok(())
}
#[sqlx::test]
async fn concurrent_duplicate_name_has_only_one_committed_operation(
    pool: PgPool,
) -> anyhow::Result<()> {
    user(&pool, "u").await?;
    let cmd = command(&pool);
    let (a, b) = tokio::join!(
        cmd.create("u", input(&["A"])),
        cmd.create("u", input(&["A"]))
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let failed = if a.is_err() { a } else { b };
    assert!(matches!(failed, Err(UseCaseError::AuthorNameConflict(_))));
    assert_eq!(counts(&pool).await?, (1, 1, 1, 1, 1, 1));
    Ok(())
}
