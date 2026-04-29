# Add Change History with Changeset Grouping for Book and Author

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds.

This document must be maintained in accordance with `.agent/PLANS.md`.

## Purpose / Big Picture

Currently, updating or deleting a book or author permanently destroys the
previous state. There is no way to recover what was there before.

After this change, every update and delete operation records a snapshot of the
entity's state immediately before the change. Multiple related changes (for
example, a future "bulk update all unread books") are grouped under a single
**changeset**, so the entire batch can be identified and reversed as a unit.

Users gain two new capabilities:

- **View history**: query all past snapshots of a specific book or author.
- **Restore**: replay any snapshot back into the live table, making the entity
  look exactly as it did at that moment in time.

To see it working after implementation: create a book, update its title, then
call the `bookHistory` GraphQL query. One entry should appear. Call
`restoreBook` with that entry's ID; the title reverts. Run
`cargo test` and the E2E suite — all tests must pass.

## Progress

- [ ] Milestone 1: Database migration — new tables
- [ ] Milestone 2: Domain layer — new entities and repository traits
- [ ] Milestone 3: Infrastructure layer — Pg implementations
- [ ] Milestone 4: Use case layer — interactors, DTOs, traits
- [ ] Milestone 5: Presentation layer — GraphQL queries and mutations
- [ ] Milestone 6: Dependency injection wiring
- [ ] Milestone 7: Unit tests
- [ ] Milestone 8: E2E tests

## Surprises & Discoveries

*(Fill in as work proceeds.)*

## Decision Log

- Decision: Use a `change_set` table to group related changes, with FK
  references from `book_history` and `author_history`.
  Rationale: Enables bulk-update operations in the future to share a single
  changeset ID so the whole batch can be restored atomically. Simple history
  tables alone cannot express this grouping.
  Date/Author: 2026-04-29 / hiterm

- Decision: Write changeset row first; write history and apply the data change
  in a single PostgreSQL transaction inside the infrastructure repository.
  Rationale: The interactor creates the changeset (via `ChangeSetRepository`),
  then passes the `ChangeSetId` to `book_repository.update` /
  `book_repository.delete`. The Pg implementation wraps the history INSERT and
  the data UPDATE/DELETE in one `BEGIN … COMMIT`. This gives atomicity without
  leaking transaction handles through the domain trait boundary.
  Date/Author: 2026-04-29 / hiterm

- Decision: Modify the existing `BookRepository::update`, `BookRepository::delete`,
  `AuthorRepository::update`, and `AuthorRepository::delete` trait methods to
  accept an additional `change_set_id: &ChangeSetId` parameter.
  Rationale: Makes history recording mandatory on every mutation. Any
  infrastructure that implements the trait is forced to handle it. The
  alternative of separate `update_with_history` methods would allow callers to
  silently bypass history.
  Date/Author: 2026-04-29 / hiterm

- Decision: Restore is itself audited. Before overwriting live data, the restore
  interactor creates a new changeset (operation = "restore") and writes the
  current live state to history, then applies the snapshot.
  Rationale: This makes every state transition reversible, including restores.
  Date/Author: 2026-04-29 / hiterm

- Decision: `Author` entity currently lacks `yomi`, `created_at`, and
  `updated_at`. The `author_history` table stores `yomi` and timestamps fetched
  directly from the DB row. The domain `Author` entity is NOT extended as part
  of this plan — that is a separate concern.
  Rationale: Scope containment. Fetching raw DB fields for the history snapshot
  is acceptable at the infrastructure layer.
  Date/Author: 2026-04-29 / hiterm

## Outcomes & Retrospective

*(Fill in at completion.)*

## Context and Orientation

This repository is a Rust/async-graphql API backed by PostgreSQL (via sqlx).
It follows a strict layered architecture:

- **Domain layer** (`src/domain/`): Pure Rust entities and repository traits.
  No database access. Repository traits use `#[automock]` (mockall) so unit
  tests can inject mocks.
- **Infrastructure layer** (`src/infrastructure/`): Concrete `Pg*Repository`
  structs that implement the domain traits using `sqlx::Pool<Postgres>`.
- **Use case layer** (`src/use_case/`): Interactors (business logic), DTOs,
  and use-case traits. Interactors depend only on domain repository traits.
- **Presentation layer** (`src/presentation/graphql/`): async-graphql schema,
  resolvers, and input/output types.
- **Dependency injection** (`src/dependency_injection.rs`): assembles
  everything for production use.

Key existing files:

    src/domain/entity/book.rs          — Book, BookId, BookTitle, …
    src/domain/entity/author.rs        — Author, AuthorId, AuthorName
    src/domain/repository/book_repository.rs   — BookRepository trait
    src/domain/repository/author_repository.rs — AuthorRepository trait
    src/infrastructure/book_repository.rs      — PgBookRepository
    src/infrastructure/author_repository.rs    — PgAuthorRepository
    src/use_case/interactor/book.rs    — UpdateBookInteractor, DeleteBookInteractor, …
    src/use_case/interactor/author.rs  — UpdateAuthorInteractor, DeleteAuthorInteractor, …
    src/use_case/interactor/mutation.rs — MutationInteractor (facade)
    src/presentation/graphql/mutation.rs       — GraphQL mutation resolvers
    src/presentation/graphql/query.rs          — GraphQL query resolvers
    src/dependency_injection.rs        — wires all Pg repos and interactors
    migrations/20220306122339_create_tables.sql — current schema

The single existing migration creates tables `bookshelf_user`, `book`,
`author`, `book_author`, `book_format`, and `book_store`.

**Terminology used in this plan:**

- *Changeset*: a record that groups one or more history snapshots created by a
  single logical operation (e.g. one update, or one future bulk-update).
- *History entry / snapshot*: a copy of an entity's state captured immediately
  before it was mutated or deleted.
- *Restore*: copying a history snapshot back into the live entity table.

## Plan of Work

### Milestone 1 — Database Migration

Create a new migration file at
`migrations/<timestamp>_add_change_history.sql`. Use `date +%Y%m%d%H%M%S` to
generate the timestamp prefix at the moment of creation.

The migration creates four tables:

`change_set` — one row per logical operation.

    CREATE TABLE change_set (
      id          uuid        NOT NULL PRIMARY KEY,
      user_id     text        NOT NULL REFERENCES bookshelf_user(id),
      operation   text        NOT NULL,
      created_at  timestamptz NOT NULL DEFAULT current_timestamp
    );

The `operation` column is a free-form label such as `"update_book"`,
`"delete_book"`, `"update_author"`, `"delete_author"`, `"restore_book"`,
`"restore_author"`. It is stored as plain text — no enum type — to allow new
values without additional migrations.

`book_history` — snapshot of a book before it was changed.

    CREATE TABLE book_history (
      history_id      bigserial   NOT NULL PRIMARY KEY,
      change_set_id   uuid        NOT NULL REFERENCES change_set(id),
      operation       text        NOT NULL,   -- 'update' or 'delete'
      book_id         uuid        NOT NULL,
      user_id         text        NOT NULL,
      title           text        NOT NULL,
      isbn            text        NOT NULL,
      read            boolean     NOT NULL,
      owned           boolean     NOT NULL,
      priority        integer     NOT NULL,
      format          text        NOT NULL,
      store           text        NOT NULL,
      book_created_at timestamptz NOT NULL,
      book_updated_at timestamptz NOT NULL,
      changed_at      timestamptz NOT NULL DEFAULT current_timestamp
    );

`book_history_author` — author IDs associated with the book at snapshot time.

    CREATE TABLE book_history_author (
      history_id  bigint NOT NULL REFERENCES book_history(history_id) ON DELETE CASCADE,
      author_id   uuid   NOT NULL,
      PRIMARY KEY (history_id, author_id)
    );

`author_history` — snapshot of an author before it was changed.

    CREATE TABLE author_history (
      history_id        bigserial   NOT NULL PRIMARY KEY,
      change_set_id     uuid        NOT NULL REFERENCES change_set(id),
      operation         text        NOT NULL,
      author_id         uuid        NOT NULL,
      user_id           text        NOT NULL,
      name              text        NOT NULL,
      yomi              text        NOT NULL,
      author_created_at timestamptz NOT NULL,
      author_updated_at timestamptz NOT NULL,
      changed_at        timestamptz NOT NULL DEFAULT current_timestamp
    );

Add indexes to support the most common queries:

    CREATE INDEX ON book_history (user_id, book_id, changed_at DESC);
    CREATE INDEX ON author_history (user_id, author_id, changed_at DESC);
    CREATE INDEX ON book_history (change_set_id);
    CREATE INDEX ON author_history (change_set_id);

At the end of this milestone run `sqlx migrate run` (requires `DATABASE_URL`
set) to verify the migration applies cleanly.

### Milestone 2 — Domain Layer

**New entity file `src/domain/entity/change_set.rs`**

Define:

    pub struct ChangeSetId { id: Uuid }
    // impl new, to_uuid, Display, TryFrom<&str>

    pub struct ChangeSet {
        id: ChangeSetId,
        user_id: UserId,
        operation: String,
        created_at: OffsetDateTime,
    }
    // pub getters via getset

**New entity file `src/domain/entity/history.rs`**

Define `HistoryOperation` as an enum:

    pub enum HistoryOperation { Update, Delete }

Define `BookHistory`:

    pub struct BookHistory {
        pub history_id: i64,
        pub change_set_id: ChangeSetId,
        pub operation: HistoryOperation,
        pub book_id: BookId,
        pub title: BookTitle,
        pub author_ids: Vec<AuthorId>,
        pub isbn: Isbn,
        pub read: ReadFlag,
        pub owned: OwnedFlag,
        pub priority: Priority,
        pub format: BookFormat,
        pub store: BookStore,
        pub book_created_at: OffsetDateTime,
        pub book_updated_at: OffsetDateTime,
        pub changed_at: OffsetDateTime,
    }

Define `AuthorHistory`:

    pub struct AuthorHistory {
        pub history_id: i64,
        pub change_set_id: ChangeSetId,
        pub operation: HistoryOperation,
        pub author_id: AuthorId,
        pub name: String,
        pub yomi: String,
        pub author_created_at: OffsetDateTime,
        pub author_updated_at: OffsetDateTime,
        pub changed_at: OffsetDateTime,
    }

Update `src/domain/entity.rs` to declare the two new modules:

    pub mod change_set;
    pub mod history;

**Modify `src/domain/repository/book_repository.rs`**

Add `change_set_id: &ChangeSetId` as the third argument to `update` and
`delete`. The updated trait signatures are:

    async fn update(
        &self,
        user_id: &UserId,
        book: &Book,
        change_set_id: &ChangeSetId,
    ) -> Result<(), DomainError>;

    async fn delete(
        &self,
        user_id: &UserId,
        book_id: &BookId,
        change_set_id: &ChangeSetId,
    ) -> Result<(), DomainError>;

**Modify `src/domain/repository/author_repository.rs`** — same pattern for
`update` and `delete`.

**New repository trait file
`src/domain/repository/change_set_repository.rs`**

    #[automock]
    #[async_trait]
    pub trait ChangeSetRepository: Send + Sync + 'static {
        async fn create(
            &self,
            user_id: &UserId,
            operation: &str,
        ) -> Result<ChangeSet, DomainError>;
    }

**New repository trait file
`src/domain/repository/book_history_repository.rs`**

    #[automock]
    #[async_trait]
    pub trait BookHistoryRepository: Send + Sync + 'static {
        async fn find_by_book(
            &self,
            user_id: &UserId,
            book_id: &BookId,
        ) -> Result<Vec<BookHistory>, DomainError>;
    }

**New repository trait file
`src/domain/repository/author_history_repository.rs`**

    #[automock]
    #[async_trait]
    pub trait AuthorHistoryRepository: Send + Sync + 'static {
        async fn find_by_author(
            &self,
            user_id: &UserId,
            author_id: &AuthorId,
        ) -> Result<Vec<AuthorHistory>, DomainError>;
    }

Update `src/domain/repository.rs` to declare all three new modules:

    pub mod change_set_repository;
    pub mod book_history_repository;
    pub mod author_history_repository;

At the end of this milestone, `cargo build` must succeed (fix all compilation
errors from changed trait signatures).

### Milestone 3 — Infrastructure Layer

**Modify `src/infrastructure/book_repository.rs` (`PgBookRepository`)**

The `update` method now receives `change_set_id`. Implement it as a single
PostgreSQL transaction:

    BEGIN;
      -- 1. Fetch current book (with author IDs via book_author)
      -- 2. INSERT INTO book_history (...) VALUES (...) RETURNING history_id
      -- 3. INSERT INTO book_history_author (history_id, author_id) for each author
      -- 4. UPDATE book SET ... WHERE id = $1 AND user_id = $2
    COMMIT;

Use `pool.begin().await?` to obtain a `sqlx::Transaction<Postgres>`, and
`.commit().await?` at the end. If any step errors, the transaction rolls back
automatically on drop.

The `delete` method follows the same pattern:

    BEGIN;
      -- 1. INSERT INTO book_history (operation='delete', ...)
      -- 2. INSERT INTO book_history_author ...
      -- 3. DELETE FROM book_author WHERE ...
      -- 4. DELETE FROM book WHERE ...
    COMMIT;

**Modify `src/infrastructure/author_repository.rs` (`PgAuthorRepository`)**

Same transaction pattern for `update` and `delete`. The `author_history`
INSERT must read `yomi` and timestamps from the DB row before changing it (one
SELECT before UPDATE/DELETE within the transaction).

**New file `src/infrastructure/change_set_repository.rs`
(`PgChangeSetRepository`)**

    pub struct PgChangeSetRepository { pool: Pool<Postgres> }

    impl PgChangeSetRepository {
        pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } }
    }

Implement `ChangeSetRepository`:

    async fn create(&self, user_id: &UserId, operation: &str) -> Result<ChangeSet, DomainError> {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO change_set (id, user_id, operation) VALUES ($1, $2, $3)",
            id, user_id.as_str(), operation
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        // construct and return ChangeSet
    }

**New file `src/infrastructure/book_history_repository.rs`
(`PgBookHistoryRepository`)**

Implements `BookHistoryRepository::find_by_book`: SELECT all rows from
`book_history` (with a LEFT JOIN to `book_history_author`) for a given
`(user_id, book_id)`, ordered by `changed_at DESC`.

**New file `src/infrastructure/author_history_repository.rs`
(`PgAuthorHistoryRepository`)**

Implements `AuthorHistoryRepository::find_by_author`: SELECT all rows from
`author_history` for a given `(user_id, author_id)`, ordered by `changed_at
DESC`.

Update `src/infrastructure.rs` to declare all new modules.

After this milestone, `cargo build` must succeed and `cargo test` must pass
(existing unit tests use mocks so changing trait signatures requires updating
the mock expectations — `#[automock]` regenerates them automatically from the
new trait definition).

### Milestone 4 — Use Case Layer

**DTO file `src/use_case/dto/history.rs`**

    pub struct BookHistoryDto {
        pub history_id: i64,
        pub change_set_id: String,
        pub operation: String,   // "update" | "delete"
        pub book_id: String,
        pub title: String,
        pub author_ids: Vec<String>,
        pub isbn: String,
        pub read: bool,
        pub owned: bool,
        pub priority: i32,
        pub format: BookFormat,
        pub store: BookStore,
        pub book_created_at: OffsetDateTime,
        pub book_updated_at: OffsetDateTime,
        pub changed_at: OffsetDateTime,
    }

    pub struct AuthorHistoryDto {
        pub history_id: i64,
        pub change_set_id: String,
        pub operation: String,
        pub author_id: String,
        pub name: String,
        pub yomi: String,
        pub author_created_at: OffsetDateTime,
        pub author_updated_at: OffsetDateTime,
        pub changed_at: OffsetDateTime,
    }

Implement `From<BookHistory>` for `BookHistoryDto` and
`From<AuthorHistory>` for `AuthorHistoryDto`.

Update `src/use_case/dto.rs` to declare the `history` module.

**New use case trait file `src/use_case/traits/history.rs`**

    #[automock]
    #[async_trait]
    pub trait ListBookHistoryUseCase: Send + Sync + 'static {
        async fn list(
            &self,
            user_id: &str,
            book_id: &str,
        ) -> Result<Vec<BookHistoryDto>, UseCaseError>;
    }

    #[automock]
    #[async_trait]
    pub trait ListAuthorHistoryUseCase: Send + Sync + 'static {
        async fn list(
            &self,
            user_id: &str,
            author_id: &str,
        ) -> Result<Vec<AuthorHistoryDto>, UseCaseError>;
    }

    #[automock]
    #[async_trait]
    pub trait RestoreBookUseCase: Send + Sync + 'static {
        async fn restore(
            &self,
            user_id: &str,
            history_id: i64,
        ) -> Result<BookDto, UseCaseError>;
    }

    #[automock]
    #[async_trait]
    pub trait RestoreAuthorUseCase: Send + Sync + 'static {
        async fn restore(
            &self,
            user_id: &str,
            history_id: i64,
        ) -> Result<AuthorDto, UseCaseError>;
    }

Update `src/use_case/traits.rs` to declare the `history` module.

**Modify `src/use_case/interactor/book.rs`**

`UpdateBookInteractor` and `DeleteBookInteractor` gain a `change_set_repository`
field:

    pub struct UpdateBookInteractor<BR, CSR> {
        book_repository: BR,
        change_set_repository: CSR,
    }

In `UpdateBookInteractor::update`:

    1. let change_set = self.change_set_repository.create(&user_id, "update_book").await?;
    2. (existing logic to build updated Book)
    3. self.book_repository.update(&user_id, &book, change_set.id()).await?;

In `DeleteBookInteractor::delete`:

    1. let change_set = self.change_set_repository.create(&user_id, "delete_book").await?;
    2. self.book_repository.delete(&user_id, &book_id, change_set.id()).await?;

**Modify `src/use_case/interactor/author.rs`** — same pattern.

**New file `src/use_case/interactor/history.rs`**

`ListBookHistoryInteractor<BHR>` where `BHR: BookHistoryRepository`:

    async fn list(&self, user_id: &str, book_id: &str) -> Result<Vec<BookHistoryDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        let entries = self.book_history_repository.find_by_book(&user_id, &book_id).await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

`ListAuthorHistoryInteractor<AHR>` — same pattern.

`RestoreBookInteractor<BR, BHR, CSR>`:

    async fn restore(&self, user_id: &str, history_id: i64) -> Result<BookDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        // 1. Find history entry (SELECT from book_history WHERE history_id = $1 AND user_id = $2)
        let snapshot = self.book_history_repository
            .find_by_history_id(&user_id, history_id).await?
            .ok_or_else(|| UseCaseError::NotFound { ... })?;
        // 2. Create "restore" changeset
        let change_set = self.change_set_repository.create(&user_id, "restore_book").await?;
        // 3. Build Book from snapshot fields
        let book = Book::new(snapshot.book_id, snapshot.title, ...)?;
        // 4. Update (records current state to history first, then overwrites)
        self.book_repository.update(&user_id, &book, change_set.id()).await?;
        Ok(book.into())
    }

Add `find_by_history_id` to `BookHistoryRepository` trait and its mock:

    async fn find_by_history_id(
        &self,
        user_id: &UserId,
        history_id: i64,
    ) -> Result<Option<BookHistory>, DomainError>;

Add the equivalent `find_by_history_id` to `AuthorHistoryRepository` for
`RestoreAuthorInteractor`.

Update `src/use_case/interactor.rs` to declare the `history` module.

**Add history use cases to `QueryInteractor`
(`src/use_case/interactor/query.rs`)**

`QueryInteractor` already delegates book/author queries. Add:

    pub book_history_repository: BHR,
    pub author_history_repository: AHR,

and delegate `list_book_history` / `list_author_history` calls.

Because `QueryInteractor` implements a `QueryUseCase` trait
(`src/use_case/traits/query.rs`), add the two list-history methods there as
well.

**Modify `src/use_case/interactor/mutation.rs` (`MutationInteractor`)**

Add two more type parameters and fields:

    restore_book_use_case: RBUC,
    restore_author_use_case: RAUC,

Add `restore_book` and `restore_author` to `MutationUseCase` trait
(`src/use_case/traits/mutation.rs`) and implement delegation.

After this milestone, `cargo test` must pass.

### Milestone 5 — Presentation Layer

**New GraphQL output types in `src/presentation/graphql/object.rs`**

    #[derive(SimpleObject)]
    pub struct BookHistoryEntry {
        pub history_id: ID,
        pub change_set_id: ID,
        pub operation: String,
        pub book_id: ID,
        pub title: String,
        pub author_ids: Vec<ID>,
        pub isbn: String,
        pub read: bool,
        pub owned: bool,
        pub priority: i32,
        pub format: String,
        pub store: String,
        pub book_created_at: String,   // ISO 8601
        pub book_updated_at: String,
        pub changed_at: String,
    }

    #[derive(SimpleObject)]
    pub struct AuthorHistoryEntry {
        pub history_id: ID,
        pub change_set_id: ID,
        pub operation: String,
        pub author_id: ID,
        pub name: String,
        pub yomi: String,
        pub author_created_at: String,
        pub author_updated_at: String,
        pub changed_at: String,
    }

Implement `From<BookHistoryDto>` for `BookHistoryEntry` and
`From<AuthorHistoryDto>` for `AuthorHistoryEntry`.

**Add queries in `src/presentation/graphql/query.rs`**

    async fn book_history(
        &self,
        ctx: &Context<'_>,
        book_id: ID,
    ) -> Result<Vec<BookHistoryEntry>, PresentationalError> { ... }

    async fn author_history(
        &self,
        ctx: &Context<'_>,
        author_id: ID,
    ) -> Result<Vec<AuthorHistoryEntry>, PresentationalError> { ... }

**Add mutations in `src/presentation/graphql/mutation.rs`**

    async fn restore_book(
        &self,
        ctx: &Context<'_>,
        history_id: ID,
    ) -> Result<Book, PresentationalError> {
        let history_id: i64 = history_id.parse().map_err(...)?;
        ...
    }

    async fn restore_author(
        &self,
        ctx: &Context<'_>,
        history_id: ID,
    ) -> Result<Author, PresentationalError> { ... }

Regenerate `schema.graphql` by running:

    cargo run --bin gen_schema > schema.graphql

### Milestone 6 — Dependency Injection

Update `src/dependency_injection.rs` to instantiate and wire up all new
repositories and interactors. The new type aliases `QI` and `MI` must reflect
the added generic parameters.

New Pg repositories to instantiate:

    let change_set_repository = PgChangeSetRepository::new(pool.clone());
    let book_history_repository = PgBookHistoryRepository::new(pool.clone());
    let author_history_repository = PgAuthorHistoryRepository::new(pool.clone());

Pass `change_set_repository` into `UpdateBookInteractor`, `DeleteBookInteractor`,
`UpdateAuthorInteractor`, `DeleteAuthorInteractor`.

Add `RestoreBookInteractor` and `RestoreAuthorInteractor` to `MutationInteractor`.

Add `book_history_repository` and `author_history_repository` to `QueryInteractor`.

### Milestone 7 — Unit Tests

For each new interactor add a `#[cfg(test)]` module using `mockall`-generated
mocks, following the pattern in `src/use_case/interactor/author.rs`.

Required new tests:

- `update_book_creates_changeset_and_calls_update_with_change_set_id` —
  verifies that `ChangeSetRepository::create` is called exactly once and the
  returned `ChangeSetId` is forwarded to `BookRepository::update`.
- `delete_book_creates_changeset` — same for delete.
- `update_author_creates_changeset` — same for author.
- `delete_author_creates_changeset` — same for author.
- `list_book_history_returns_dto_list` — happy path.
- `list_book_history_returns_empty_when_none` — empty list.
- `list_author_history_returns_dto_list` — happy path.
- `restore_book_not_found_returns_error` — `find_by_history_id` returns None.
- `restore_book_success` — snapshot loaded, changeset created, update called.
- `restore_author_success` — same for author.

Existing tests for `UpdateBookInteractor` and `DeleteBookInteractor` must be
updated to also set expectations on `ChangeSetRepository`. Because the trait
now uses `#[automock]`, the generated `MockChangeSetRepository` is available
automatically.

Run `cargo test` and confirm all tests pass.

### Milestone 8 — E2E Tests

The E2E test suite lives in `e2e/`. Examine the existing test files to
understand the pattern (likely TypeScript/JavaScript using a GraphQL client).

Add a new test file (or extend an existing book test file) covering:

1. Create a book → update its title → call `bookHistory` → expect one entry
   with `operation = "update"` and the old title.
2. Restore using `restoreBook(historyId)` → fetch the book → expect the old
   title is back.
3. Delete a book → call `bookHistory` → expect one entry with
   `operation = "delete"`.
4. Create an author → update its name → call `authorHistory` → expect one
   entry.
5. Restore using `restoreAuthor(historyId)` → fetch the author → expect old
   name.

Run the E2E suite against a local Docker Compose stack:

    docker compose -f docker-compose-test.yml up -d
    # run E2E tests per existing suite instructions
    docker compose -f docker-compose-test.yml down

All scenarios must pass.

## Concrete Steps

Run all commands from the repository root
(`/home/hiterm/ghq/github.com/hiterm/bookshelf-api`) unless otherwise noted.

1. Create the migration file:

       TS=$(date +%Y%m%d%H%M%S)
       touch migrations/${TS}_add_change_history.sql

   Write the SQL from Milestone 1 into that file.

2. Apply the migration (requires `DATABASE_URL`):

       sqlx migrate run

3. Implement Milestone 2 (domain layer). After each new file, run:

       cargo build 2>&1 | head -40

4. Implement Milestone 3 (infrastructure layer). Verify:

       cargo build

5. Implement Milestone 4 (use case layer). Verify:

       cargo test

6. Implement Milestone 5 (presentation layer). Regenerate schema:

       cargo run --bin gen_schema > schema.graphql

7. Implement Milestone 6 (DI). Verify full build:

       cargo build

8. Run pre-commit checks (mandatory per CLAUDE.md):

       cargo fmt --check
       cargo clippy --all-targets -- -D warnings
       cargo test

   Fix any failures before committing.

9. Commit with a descriptive message per CLAUDE.md conventions.

10. Run E2E suite (Milestone 8).

## Validation and Acceptance

Unit test acceptance: `cargo test` must report 0 failures. The new tests
listed in Milestone 7 must exist and pass.

E2E acceptance: all five E2E scenarios in Milestone 8 pass.

Manual smoke test (optional, requires running server):

    # Start server
    cargo run

    # Create a book (substitute real token and book data)
    curl -s -X POST http://localhost:8080/graphql \
      -H "Authorization: Bearer <token>" \
      -H "Content-Type: application/json" \
      -d '{"query":"mutation { createBook(bookData: {title:\"Foo\", ...}) { id } }"}'

    # Update the book title
    # Query bookHistory — expect one entry
    # Restore — expect title reverts

## Idempotence and Recovery

The migration uses `CREATE TABLE` without `IF NOT EXISTS`. Running it twice
will fail. sqlx tracks applied migrations in `_sqlx_migrations` and will not
re-apply them, so this is safe.

If a milestone fails to compile, fix the compilation errors before proceeding.
Do not commit broken code.

## Interfaces and Dependencies

In `src/domain/repository/change_set_repository.rs`:

    pub trait ChangeSetRepository: Send + Sync + 'static {
        async fn create(&self, user_id: &UserId, operation: &str) -> Result<ChangeSet, DomainError>;
    }

In `src/domain/repository/book_history_repository.rs`:

    pub trait BookHistoryRepository: Send + Sync + 'static {
        async fn find_by_book(&self, user_id: &UserId, book_id: &BookId)
            -> Result<Vec<BookHistory>, DomainError>;
        async fn find_by_history_id(&self, user_id: &UserId, history_id: i64)
            -> Result<Option<BookHistory>, DomainError>;
    }

In `src/domain/repository/author_history_repository.rs`:

    pub trait AuthorHistoryRepository: Send + Sync + 'static {
        async fn find_by_author(&self, user_id: &UserId, author_id: &AuthorId)
            -> Result<Vec<AuthorHistory>, DomainError>;
        async fn find_by_history_id(&self, user_id: &UserId, history_id: i64)
            -> Result<Option<AuthorHistory>, DomainError>;
    }

In `src/domain/repository/book_repository.rs` (modified signatures):

    async fn update(&self, user_id: &UserId, book: &Book, change_set_id: &ChangeSetId)
        -> Result<(), DomainError>;
    async fn delete(&self, user_id: &UserId, book_id: &BookId, change_set_id: &ChangeSetId)
        -> Result<(), DomainError>;

In `src/domain/repository/author_repository.rs` (modified signatures):

    async fn update(&self, user_id: &UserId, author: &Author, change_set_id: &ChangeSetId)
        -> Result<(), DomainError>;
    async fn delete(&self, user_id: &UserId, author_id: &AuthorId, change_set_id: &ChangeSetId)
        -> Result<(), DomainError>;
