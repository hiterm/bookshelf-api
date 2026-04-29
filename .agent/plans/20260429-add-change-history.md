# Add Change History with Changeset Grouping for Book and Author

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds.

This document must be maintained in accordance with `.agent/PLANS.md`.

**Plan update rule**: Update this document continuously as work proceeds —
mark each task done the moment it is completed, record discoveries immediately
when found, and log decisions as soon as they are made. Do NOT batch updates
and apply them all at the end.

**Commit granularity rule**: Commit at each logical breakpoint — completing a
migration file, adding a new entity, implementing a repository method, adding
a test suite, and so on. Do not batch unrelated changes into one commit. Each
commit message must describe what specifically changed and why.

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

- [x] Milestone 1: Database migration — new tables
- [x] Milestone 2: Domain layer — new entities and repository traits
- [x] Milestone 3: Infrastructure layer — Pg implementations
- [x] Milestone 4: Use case layer — interactors, DTOs, traits
- [x] Milestone 5: Presentation layer — GraphQL queries and mutations
- [x] Milestone 6: Dependency injection wiring
- [x] Milestone 7: Unit tests
- [x] Milestone 8: E2E tests

## Surprises & Discoveries

- **Duplicate `ChangeSetId` definition**: Initially defined `ChangeSetId` inside
  `history.rs` as well as in `change_set.rs`. The compiler caught the conflict
  when both were imported in the same file. Fixed by removing the duplicate from
  `history.rs` and importing from `change_set.rs`.

- **`QueryInteractor` arity mismatch**: After adding `BHR` and `AHR` type params
  to `QueryInteractor`, `dependency_injection.rs` still had the 3-param form and
  the struct literal was missing the two new fields. All 13 test struct
  constructions also had to be updated (`replace_all=true`).

- **`cargo clippy --fix` requires `--allow-dirty`**: The tool refuses to apply
  fixes when the working tree has uncommitted changes. Must pass `--allow-dirty`
  when running as part of the pre-commit flow before staging.

- **`OffsetDateTime` import missing in test module**: The `use time::OffsetDateTime`
  import existed at the top-level module but was not in scope inside the nested
  `#[cfg(test)]` mod. Had to add a separate `use time::OffsetDateTime` inside
  the test module.

- **`cargo fmt` reformats `clippy --fix` output**: After `clippy --fix` auto-fixed
  files, `cargo fmt --check` still reported diffs. Running `cargo fmt` a second
  time after clippy was required to produce a clean check.

- **`create` operations are also snapshotted**: The plan recorded history only
  for update/delete, but snapshotting create makes `bookHistory`/`authorHistory`
  always return at least one entry and allows `restoreBook`/`restoreAuthor` to
  be used to undo accidental creates. The decision was recorded in the Decision
  Log during implementation.

## Decision Log

- Decision: Use a `change_set` table to group related changes, with FK
  references from `book_history` and `author_history`.
  Rationale: Enables bulk-update operations in the future to share a single
  changeset ID so the whole batch can be restored atomically. Simple history
  tables alone cannot express this grouping.
  Date/Author: 2026-04-29 / hiterm

- Decision: The changeset UUID is generated inside the infrastructure repository
  (`PgBookRepository`, `PgAuthorRepository`), not by the interactor. The
  entire sequence — generate changeset UUID, INSERT into `change_set`, INSERT
  into `*_history`, apply the data change — is wrapped in a single PostgreSQL
  transaction (`BEGIN … COMMIT`).
  Rationale: Keeps `BookRepository` and `AuthorRepository` trait signatures
  unchanged. Interactors stay simple; they call `update`/`delete` exactly as
  before. When a future bulk-update API is added to bookshelf-api, a new
  repository method (e.g. `bulk_update`) will generate one shared changeset
  for all items in that call.
  Date/Author: 2026-04-29 / hiterm

- Decision: Do NOT add `change_set_id` to `BookRepository::update/delete` or
  `AuthorRepository::update/delete`. Do NOT introduce a `ChangeSetRepository`
  domain trait. Changeset creation is an infrastructure-only concern for
  single-item operations.
  Rationale: The user confirmed that bulk updates will be implemented as a
  dedicated API in bookshelf-api. A future `bulk_update` repository method can
  encapsulate one changeset for all items without changing existing signatures.
  Avoiding signature changes means zero churn in interactors and their unit
  tests.
  Date/Author: 2026-04-29 / hiterm

- Decision: Restore is itself audited. Before overwriting live data, the restore
  interactor creates a new changeset (operation = "restore") and writes the
  current live state to history, then applies the snapshot.
  Rationale: This makes every state transition reversible, including restores.
  Date/Author: 2026-04-29 / hiterm

- Decision: Record history on create as well as update and delete, using
  operation='create' in both `change_set` and `book_history`/`author_history`.
  Rationale: The user wants to preserve the fact that an item was created (who
  added it and when). Unlike update/delete, there is no previous state to
  snapshot — the history row IS the initial state. This means `bookHistory`
  will always show at least one entry for every book.
  Date/Author: 2026-04-29 / hiterm

- Decision: `Author` entity currently lacks `yomi`, `created_at`, and
  `updated_at`. The `author_history` table stores `yomi` and timestamps fetched
  directly from the DB row. The domain `Author` entity is NOT extended as part
  of this plan — that is a separate concern.
  Rationale: Scope containment. Fetching raw DB fields for the history snapshot
  is acceptable at the infrastructure layer.
  Date/Author: 2026-04-29 / hiterm

## Outcomes & Retrospective

All 8 milestones completed on 2026-04-29.

**What was built:**
- Migration: `change_set`, `book_history`, `book_history_author`, `author_history`
  tables with 4 indexes.
- Domain: `ChangeSetId`, `ChangeSet`, `HistoryOperation`, `BookHistory`,
  `AuthorHistory` entities; `BookHistoryRepository` and `AuthorHistoryRepository`
  traits with `#[automock]`.
- Infrastructure: `PgBookHistoryRepository` and `PgAuthorHistoryRepository`;
  `PgBookRepository` and `PgAuthorRepository` fully wrapped in transactions that
  snapshot state on every create/update/delete.
- Use case: `BookHistoryDto`, `AuthorHistoryDto`; `ListBookHistoryInteractor`,
  `ListAuthorHistoryInteractor`, `RestoreBookInteractor`, `RestoreAuthorInteractor`;
  four new use case traits all with `#[automock]`.
- Presentation: `BookHistoryEntry`, `AuthorHistoryEntry` GraphQL types;
  `bookHistory`/`authorHistory` queries; `restoreBook`/`restoreAuthor` mutations.
- DI: updated type aliases `QI` and `MI` with 5 and 9 type params respectively.
- Unit tests: 96 passing (6 new tests for history interactors).
- E2E tests: 7 new serial E2E tests covering create/update/delete history recording
  and restore for both books and authors.

**What worked well:** The strict layered architecture meant infrastructure
changes (wrapping every repo method in a transaction) were completely invisible
to interactors and their unit tests — zero churn to the 90 pre-existing tests.

**What to watch:** E2E tests for delete history rely on querying `bookHistory`
after the book is deleted. This works because `book_history` has no FK to `book`
(intentional), so history survives the delete.

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

The existing `BookRepository` and `AuthorRepository` trait signatures are
**not changed**. `update` and `delete` keep their current parameter lists.
No `ChangeSetRepository` domain trait is introduced; changeset creation is
handled entirely inside the infrastructure layer.

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

Update `src/domain/repository.rs` to declare the two new modules:

    pub mod book_history_repository;
    pub mod author_history_repository;

At the end of this milestone, `cargo build` must succeed.

### Milestone 3 — Infrastructure Layer

**Modify `src/infrastructure/book_repository.rs` (`PgBookRepository`)**

The `update` and `delete` method signatures stay the same. Inside each method,
generate a changeset UUID and wrap everything in a single transaction using
`pool.begin().await?`. Call `.commit().await?` at the end; any error causes an
automatic rollback on drop.

`create` transaction steps (new — `BookRepository::create` is also wrapped):

    BEGIN;
      -- 1. INSERT INTO book ...
      -- 2. INSERT INTO book_author ... (for each author)
      -- 3. let cs_id = Uuid::new_v4();
      -- 4. INSERT INTO change_set (id, user_id, operation='create_book', ...)
      -- 5. INSERT INTO book_history (change_set_id=cs_id, operation='create', ...)
             RETURNING history_id
      -- 6. INSERT INTO book_history_author (history_id, author_id) for each author
    COMMIT;

`update` transaction steps:

    BEGIN;
      -- 1. SELECT book + author IDs (current state snapshot)
      -- 2. let cs_id = Uuid::new_v4();
      -- 3. INSERT INTO change_set (id, user_id, operation='update_book', ...)
      -- 4. INSERT INTO book_history (change_set_id=cs_id, operation='update', ...)
             RETURNING history_id
      -- 5. INSERT INTO book_history_author (history_id, author_id) for each author
      -- 6. UPDATE book SET ... WHERE id = $1 AND user_id = $2
    COMMIT;

`delete` transaction steps:

    BEGIN;
      -- 1. SELECT book + author IDs (current state snapshot)
      -- 2. let cs_id = Uuid::new_v4();
      -- 3. INSERT INTO change_set (id, user_id, operation='delete_book', ...)
      -- 4. INSERT INTO book_history (change_set_id=cs_id, operation='delete', ...)
             RETURNING history_id
      -- 5. INSERT INTO book_history_author ...
      -- 6. DELETE FROM book_author WHERE ...
      -- 7. DELETE FROM book WHERE ...
    COMMIT;

**Modify `src/infrastructure/author_repository.rs` (`PgAuthorRepository`)**

Same transaction pattern. The `author_history` INSERT reads `yomi` and
timestamps from the DB row in the same SELECT before UPDATE/DELETE.

`create` uses `operation='create_author'`, `update` uses `operation='update_author'`,
`delete` uses `operation='delete_author'` in the `change_set` row. The create
transaction inserts the author first, then records it to `author_history`
(operation='create') in the same transaction.

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

After this milestone, `cargo build` must succeed and `cargo test` must pass.
Existing unit tests for `UpdateBookInteractor` and `DeleteBookInteractor` need
no changes because the repository trait signatures are unchanged.

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

**`src/use_case/interactor/book.rs` and `src/use_case/interactor/author.rs`**
are **not changed**. `UpdateBookInteractor`, `DeleteBookInteractor`,
`UpdateAuthorInteractor`, and `DeleteAuthorInteractor` call `update`/`delete`
exactly as before. Changeset creation is invisible to them.

**New file `src/use_case/interactor/history.rs`**

`ListBookHistoryInteractor<BHR>` where `BHR: BookHistoryRepository`:

    async fn list(&self, user_id: &str, book_id: &str) -> Result<Vec<BookHistoryDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        let entries = self.book_history_repository.find_by_book(&user_id, &book_id).await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

`ListAuthorHistoryInteractor<AHR>` — same pattern.

`RestoreBookInteractor<BR, BHR>`:

    async fn restore(&self, user_id: &str, history_id: i64) -> Result<BookDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        // 1. Find history entry (SELECT from book_history WHERE history_id = $1 AND user_id = $2)
        let snapshot = self.book_history_repository
            .find_by_history_id(&user_id, history_id).await?
            .ok_or_else(|| UseCaseError::NotFound { ... })?;
        // 2. Build Book from snapshot fields
        let book = Book::new(snapshot.book_id, snapshot.title, ...)?;
        // 3. Call update — PgBookRepository generates a new changeset internally
        //    (operation='update_book') and records the current live state to history
        //    before overwriting it.
        self.book_repository.update(&user_id, &book).await?;
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

    let book_history_repository = PgBookHistoryRepository::new(pool.clone());
    let author_history_repository = PgAuthorHistoryRepository::new(pool.clone());

No changes needed to the existing `UpdateBookInteractor`, `DeleteBookInteractor`,
`UpdateAuthorInteractor`, or `DeleteAuthorInteractor` constructors.

Add `RestoreBookInteractor` and `RestoreAuthorInteractor` to `MutationInteractor`.

Add `book_history_repository` and `author_history_repository` to `QueryInteractor`.

### Milestone 7 — Unit Tests

For each new interactor add a `#[cfg(test)]` module using `mockall`-generated
mocks, following the pattern in `src/use_case/interactor/author.rs`.

Required new tests:

- `list_book_history_returns_dto_list` — happy path.
- `list_book_history_returns_empty_when_none` — empty list.
- `list_author_history_returns_dto_list` — happy path.
- `restore_book_not_found_returns_error` — `find_by_history_id` returns None.
- `restore_book_success` — snapshot found, `BookRepository::update` called
  with the reconstructed book.
- `restore_author_success` — same for author.

Existing tests for `UpdateBookInteractor`, `DeleteBookInteractor`,
`UpdateAuthorInteractor`, and `DeleteAuthorInteractor` require **no changes**
because the repository trait signatures are unchanged.

Run `cargo test` and confirm all tests pass.

### Milestone 8 — E2E Tests

The E2E test suite lives in `e2e/`. Examine the existing test files to
understand the pattern (likely TypeScript/JavaScript using a GraphQL client).

Add a new test file (or extend an existing book test file) covering:

1. Create a book → call `bookHistory` → expect one entry with
   `operation = "create"`.
2. Update the book's title → call `bookHistory` → expect two entries; the
   second has `operation = "update"` and the pre-update title.
3. Restore using `restoreBook(historyId)` → fetch the book → expect the old
   title is back.
4. Delete a book → call `bookHistory` → expect one entry with
   `operation = "delete"`.
5. Create an author → call `authorHistory` → expect one entry with
   `operation = "create"`.
6. Update the author's name → call `authorHistory` → expect two entries.
7. Restore using `restoreAuthor(historyId)` → fetch the author → expect old
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

9. Commit at each logical breakpoint (migration file, new entity, repository
   method, test suite, etc.) with a descriptive message per CLAUDE.md
   conventions. Do not batch unrelated changes into one commit.

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

`BookRepository` and `AuthorRepository` trait signatures are **unchanged** from
the current codebase. No new parameters are added to `update` or `delete`.
