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
  - [x] plan updated
- [x] Milestone 2: Domain layer — new entities and repository traits
  - [x] plan updated
- [x] Milestone 3: Infrastructure layer — Pg implementations
  - [x] plan updated
- [x] Milestone 4: Use case layer — interactors, DTOs, traits
  - [x] plan updated
- [x] Milestone 5: Presentation layer — GraphQL queries and mutations
  - [x] plan updated
- [x] Milestone 6: Dependency injection wiring
  - [x] plan updated
- [x] Milestone 7: Unit tests
  - [x] plan updated
- [x] Milestone 8: E2E tests
  - [x] plan updated

**Phase 2 — Refactor to post-state event log** (started 2026-04-29)

- [x] Milestone 9: Merge migrations and refactor schema
  - [x] plan updated
- [x] Milestone 10: Rename domain entities and repository traits
  - [x] plan updated
- [x] Milestone 11: Refactor infrastructure — post-state recording and renamed tables
  - [x] plan updated
- [x] Milestone 12: Refactor use case — restore semantics and return types
  - [x] plan updated
- [x] Milestone 13: Refactor presentation — nullable GraphQL fields and return types
  - [x] plan updated
- [x] Milestone 14: Update all tests
  - [x] plan updated

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

## Decision Log (Phase 2)

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

## Decision Log (Phase 2) — Event Naming

- Decision: Rename tables from history-based to event-based naming:
  `change_set` → `event_set`, `book_history` → `book_event`,
  `book_history_author` → `book_event_author`, `author_history` → `author_event`.
  Also rename `change_set_operation` → `event_set_operation`. Keep
  `history_operation` unchanged.
  Rationale: The tables record facts (events that occurred), not snapshots.
  "History" implies retrospective snapshots; "event" accurately describes a
  record of what happened. `book_event_author` is a subordinate detail table
  of `book_event`, so the `book_event_` prefix is correct. `history_operation`
  enumerates values for the per-event operation column (create/update/delete),
  which still fits the name.
  Date/Author: 2026-04-29 / hiterm

- Decision: Switch from pre-state recording to post-state recording for
  `update` events. `delete` events record only the entity id (data fields
  nullable). `create` events are unchanged (already record post-create state).
  Rationale: Pre-state recording is semantically inconsistent — the `operation`
  column says what happened, but the data is the state *before* it happened. A
  post-state event log records the fact truthfully: "this operation happened,
  this is the resulting state." For `delete`, there is no resulting state;
  only the id is stored as the event fact. The id is sufficient to identify
  what was deleted.
  Date/Author: 2026-04-29 / hiterm

- Decision: `restore` semantics change from "apply this snapshot" to "restore
  to the state captured in this event": `create`/`update` events → apply the
  stored data (update if the entity exists, create if it was deleted);
  `delete` events → delete the entity (treat a NotFound as a no-op success).
  The restore return type changes from `BookDto`/`AuthorDto` to
  `Option<BookDto>`/`Option<AuthorDto>` (None when restoring a delete event).
  Rationale: Consistent with the event log model. Each event represents a
  specific state; restoring to it means making the world match that state.
  A delete event's state is "entity does not exist", so restore = delete.
  Date/Author: 2026-04-29 / hiterm

- Decision: `author restore` adds a create fallback identical to the existing
  book restore fallback: if `update` returns `NotFound`, call `create` instead.
  Rationale: An author may have been deleted after the `create`/`update` event
  was recorded. Without the fallback, restoring such an event would silently
  fail with NotFound.
  Date/Author: 2026-04-29 / hiterm

- Decision: The two existing migration files from Phase 1
  (`20260429040611_add_change_history.sql` and
  `20260429050000_add_operation_constraints.sql`) are merged into a single
  migration file, and the Phase 2 schema changes (table renames, nullable
  columns) are included in that same file. The two old files are deleted.
  Rationale: Both files were added in this PR and have never been applied to
  a production database, so consolidation carries no migration risk. One file
  is simpler to reason about.
  Date/Author: 2026-04-29 / hiterm

- Decision: The `event_set` table is kept (not removed), with a 1:N
  relationship to event entries. Currently it is 1:1, but it is the intended
  grouping mechanism for future bulk-update and author-merge operations.
  Rationale: Bulk registration and author merging (merging two author records
  into one) require grouping many book and author events under a single logical
  user action. Without `event_set`, those events would be unrelated rows with
  no shared identity.
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

---

## Phase 2 — Refactor to Post-State Event Log

### Overview

Phase 1 built a history system that records the *pre-operation* state of every
create, update, and delete. After design review, this was found to be
semantically inconsistent: the `operation` column says what happened, but the
stored data is the state *before* it happened, not after. Phase 2 corrects this
by switching to a post-state event log, renaming all tables to event-based
names, and updating restore semantics accordingly.

The user-visible change is that `bookHistory`/`authorHistory` now show the
state *after* each operation. The `restoreBook`/`restoreAuthor` mutations now
return `null` when restoring a `delete` event (because the entity no longer
exists after restoration), and restore a deleted entity's last known state when
restoring a `create` or `update` event.

### Milestone 9 — Merge Migrations and Refactor Schema

Delete the two existing Phase 1 migration files:

    migrations/20260429040611_add_change_history.sql
    migrations/20260429050000_add_operation_constraints.sql

Create one replacement file at the same path
`migrations/20260429040611_add_change_history.sql` (reusing the first
timestamp) with the following consolidated, refactored content:

    CREATE TABLE event_set (
      id          uuid        NOT NULL PRIMARY KEY,
      user_id     text        NOT NULL REFERENCES bookshelf_user(id),
      operation   text        NOT NULL,
      created_at  timestamptz NOT NULL DEFAULT current_timestamp
    );

    CREATE TABLE history_operation (
      operation text NOT NULL PRIMARY KEY
    );
    INSERT INTO history_operation VALUES ('create'), ('update'), ('delete');

    CREATE TABLE event_set_operation (
      operation text NOT NULL PRIMARY KEY
    );
    INSERT INTO event_set_operation VALUES
      ('create_book'), ('update_book'), ('delete_book'),
      ('create_author'), ('update_author'), ('delete_author');

    ALTER TABLE event_set
      ADD CONSTRAINT event_set_operation_fk
      FOREIGN KEY (operation) REFERENCES event_set_operation(operation);

    -- book_event: data fields are nullable because delete events store only id.
    CREATE TABLE book_event (
      event_id        bigserial   NOT NULL PRIMARY KEY,
      event_set_id    uuid        NOT NULL REFERENCES event_set(id),
      operation       text        NOT NULL REFERENCES history_operation(operation),
      book_id         uuid        NOT NULL,
      user_id         text        NOT NULL,
      title           text,
      isbn            text,
      read            boolean,
      owned           boolean,
      priority        integer,
      format          text,
      store           text,
      book_created_at timestamptz,
      book_updated_at timestamptz,
      changed_at      timestamptz NOT NULL DEFAULT current_timestamp
    );

    CREATE TABLE book_event_author (
      event_id  bigint NOT NULL REFERENCES book_event(event_id) ON DELETE CASCADE,
      author_id uuid   NOT NULL,
      PRIMARY KEY (event_id, author_id)
    );

    -- author_event: data fields nullable for same reason.
    CREATE TABLE author_event (
      event_id          bigserial   NOT NULL PRIMARY KEY,
      event_set_id      uuid        NOT NULL REFERENCES event_set(id),
      operation         text        NOT NULL REFERENCES history_operation(operation),
      author_id         uuid        NOT NULL,
      user_id           text        NOT NULL,
      name              text,
      yomi              text,
      author_created_at timestamptz,
      author_updated_at timestamptz,
      changed_at        timestamptz NOT NULL DEFAULT current_timestamp
    );

    CREATE INDEX ON book_event (user_id, book_id, changed_at DESC);
    CREATE INDEX ON author_event (user_id, author_id, changed_at DESC);
    CREATE INDEX ON book_event (event_set_id);
    CREATE INDEX ON author_event (event_set_id);

Verify by running `cargo test` (sqlx::test creates a fresh DB from migrations
each run) — the build must succeed and no test should fail due to the schema.

### Milestone 10 — Rename Domain Entities and Repository Traits

This milestone renames Rust types and files throughout the domain layer. No
logic changes yet — only names.

**`src/domain/entity/change_set.rs` → `src/domain/entity/event_set.rs`**

Rename:
- `ChangeSetId` → `EventSetId` (keep all impls: `new`, `to_uuid`, `Display`,
  `TryFrom<&str>`, `From<Uuid>`, `Default`)
- `ChangeSet` → `EventSet`

**`src/domain/entity.rs`**: change `pub mod change_set` to `pub mod event_set`.

**`src/domain/entity/history.rs`**

Rename:
- `BookHistory` → `BookEvent`
- `AuthorHistory` → `AuthorEvent`

Change `BookEvent` fields that are null for delete events from owned types
to `Option<T>`:

    pub struct BookEvent {
        pub event_id: i64,
        pub event_set_id: EventSetId,
        pub operation: HistoryOperation,
        pub book_id: BookId,
        // Some for create/update; None for delete:
        pub title: Option<BookTitle>,
        pub author_ids: Vec<AuthorId>,     // empty for delete (no book_event_author rows)
        pub isbn: Option<Isbn>,
        pub read: Option<ReadFlag>,
        pub owned: Option<OwnedFlag>,
        pub priority: Option<Priority>,
        pub format: Option<BookFormat>,
        pub store: Option<BookStore>,
        pub book_created_at: Option<OffsetDateTime>,
        pub book_updated_at: Option<OffsetDateTime>,
        pub changed_at: OffsetDateTime,
    }

    pub struct AuthorEvent {
        pub event_id: i64,
        pub event_set_id: EventSetId,
        pub operation: HistoryOperation,
        pub author_id: AuthorId,
        // Some for create/update; None for delete:
        pub name: Option<String>,
        pub yomi: Option<String>,
        pub author_created_at: Option<OffsetDateTime>,
        pub author_updated_at: Option<OffsetDateTime>,
        pub changed_at: OffsetDateTime,
    }

`HistoryOperation` gains a `Create` variant if not already present (it was
added in Phase 1 — verify and keep).

**`src/domain/repository/book_history_repository.rs` →
`src/domain/repository/book_event_repository.rs`**

Rename:
- Trait `BookHistoryRepository` → `BookEventRepository`
- Method parameter and return types updated: `BookHistory` → `BookEvent`,
  field `history_id` → `event_id` in the `find_by_history_id` signature.

    #[automock]
    #[async_trait]
    pub trait BookEventRepository: Send + Sync + 'static {
        async fn find_by_book(
            &self,
            user_id: &UserId,
            book_id: &BookId,
        ) -> Result<Vec<BookEvent>, DomainError>;

        async fn find_by_event_id(
            &self,
            user_id: &UserId,
            event_id: i64,
        ) -> Result<Option<BookEvent>, DomainError>;
    }

**`src/domain/repository/author_history_repository.rs` →
`src/domain/repository/author_event_repository.rs`**

Same pattern: `AuthorHistoryRepository` → `AuthorEventRepository`,
`AuthorHistory` → `AuthorEvent`, `find_by_history_id` → `find_by_event_id`.

**`src/domain/repository.rs`**: update module declarations to
`book_event_repository` and `author_event_repository`.

Verify: `cargo build` must succeed with no warnings.

### Milestone 11 — Refactor Infrastructure — Post-State Recording

This milestone changes *what data* is written to the event tables and updates
all SQL to use the new table and column names.

**`src/infrastructure/book_repository.rs`**

`create` — record the newly created book (post-create state). This is already
the correct behavior from Phase 1; only the SQL table names change:
`change_set` → `event_set`, `change_set_id` → `event_set_id`,
`history_id` → `event_id`, `book_history` → `book_event`,
`book_history_author` → `book_event_author`.

`update` — Phase 1 recorded the pre-update snapshot. Replace this with
post-update recording:

    BEGIN;
      -- 1. UPDATE book SET ... WHERE id = $N AND user_id = $1
      --    (proceed only if rows_affected == 1; return NotFound otherwise)
      -- 2. let es_id = Uuid::new_v4();
      -- 3. INSERT INTO event_set (id, user_id, operation='update_book')
      -- 4. INSERT INTO book_event
      --      (event_set_id=es_id, operation='update', book_id, user_id,
      --       title, isbn, read, owned, priority, format, store,
      --       book_created_at, book_updated_at)   ← values from `book` argument
      --    RETURNING event_id
      -- 5. INSERT INTO book_event_author (event_id, author_id) for each
      --    author_id in `book.author_ids()`
    COMMIT;

The `book` argument to `update` already contains the post-update state; use
its fields directly. Remove the pre-update SELECT snapshot query entirely.

`delete` — record only the book id (data fields null):

    BEGIN;
      -- 1. DELETE FROM book_author WHERE user_id = $1 AND book_id = $2
      -- 2. DELETE FROM book WHERE user_id = $1 AND id = $2
      --    (return NotFound if rows_affected == 0)
      -- 3. let es_id = Uuid::new_v4();
      -- 4. INSERT INTO event_set (id, user_id, operation='delete_book')
      -- 5. INSERT INTO book_event
      --      (event_set_id=es_id, operation='delete', book_id, user_id)
      --    All data fields (title, isbn, …) omitted — they default to NULL.
    COMMIT;

No `INSERT INTO book_event_author` for delete events.

**`src/infrastructure/author_repository.rs`**

Same pattern as `book_repository`. For `update`, use the `author` argument
fields directly (post-update state). For `delete`, insert only `author_id` and
`user_id`; all other columns are NULL.

**`src/infrastructure/book_history_repository.rs` →
`src/infrastructure/book_event_repository.rs`**

Rename struct `PgBookHistoryRepository` → `PgBookEventRepository`. Rename
inner `BookHistoryRow` → `BookEventRow` and update column names
(`history_id` → `event_id`, `change_set_id` → `event_set_id`). All fields
that are now nullable (`title`, `isbn`, `read`, `owned`, `priority`, `format`,
`store`, `book_created_at`, `book_updated_at`) must be `Option<T>` in the row
struct. Update all SQL to use `book_event` and `book_event_author`.

Update `find_by_book` to keep `GROUP BY event_id ORDER BY changed_at DESC`.
Rename `find_by_history_id` → `find_by_event_id`.

**`src/infrastructure/author_history_repository.rs` →
`src/infrastructure/author_event_repository.rs`**

Same renaming pattern. Nullable fields: `name`, `yomi`,
`author_created_at`, `author_updated_at`.

**`src/infrastructure.rs`**: update module declarations.

**`src/dependency_injection.rs`**: update imports and struct field names.

Verify: `cargo build && cargo test` must pass.

### Milestone 12 — Refactor Use Case — Restore Semantics and Return Types

**`src/use_case/dto/history.rs`**

Rename `BookHistoryDto` → `BookEventDto`, `AuthorHistoryDto` → `AuthorEventDto`.

Data fields that are null for delete events become `Option<T>`:

    pub struct BookEventDto {
        pub event_id: i64,
        pub event_set_id: String,
        pub operation: String,       // "create" | "update" | "delete"
        pub book_id: String,
        pub title: Option<String>,
        pub author_ids: Vec<String>, // empty for delete
        pub isbn: Option<String>,
        pub read: Option<bool>,
        pub owned: Option<bool>,
        pub priority: Option<i32>,
        pub format: Option<BookFormat>,
        pub store: Option<BookStore>,
        pub book_created_at: Option<OffsetDateTime>,
        pub book_updated_at: Option<OffsetDateTime>,
        pub changed_at: OffsetDateTime,
    }

    pub struct AuthorEventDto {
        pub event_id: i64,
        pub event_set_id: String,
        pub operation: String,
        pub author_id: String,
        pub name: Option<String>,
        pub yomi: Option<String>,
        pub author_created_at: Option<OffsetDateTime>,
        pub author_updated_at: Option<OffsetDateTime>,
        pub changed_at: OffsetDateTime,
    }

Update `From<BookEvent>` and `From<AuthorEvent>` impls accordingly.

**`src/use_case/traits/history.rs`**

Rename trait names and method parameters; update return types:

- `ListBookHistoryUseCase` → `ListBookEventUseCase` (returns `Vec<BookEventDto>`)
- `ListAuthorHistoryUseCase` → `ListAuthorEventUseCase`
- `RestoreBookUseCase::restore` returns `Result<Option<BookDto>, UseCaseError>`
- `RestoreAuthorUseCase::restore` returns `Result<Option<AuthorDto>, UseCaseError>`

**`src/use_case/interactor/history.rs`**

Rename interactors: `ListBookHistoryInteractor` → `ListBookEventInteractor`,
`ListAuthorHistoryInteractor` → `ListAuthorEventInteractor`. These are
mechanical renames — no logic change.

`RestoreBookInteractor::restore` — new logic:

    let user_id = UserId::new(user_id.to_string())?;
    let event = self.book_event_repository
        .find_by_event_id(&user_id, event_id).await?
        .ok_or(UseCaseError::NotFound { ... })?;

    match event.operation {
        HistoryOperation::Create | HistoryOperation::Update => {
            let book = Book::new(
                event.book_id,
                event.title.ok_or_else(|| /* internal error */)?,
                event.author_ids,
                event.isbn.ok_or_else(|| /* internal error */)?,
                event.read.ok_or_else(...)?,
                event.owned.ok_or_else(...)?,
                event.priority.ok_or_else(...)?,
                event.format.ok_or_else(...)?,
                event.store.ok_or_else(...)?,
                event.book_created_at.ok_or_else(...)?,
                event.book_updated_at.ok_or_else(...)?,
            )?;
            match self.book_repository.update(&user_id, &book).await {
                Ok(()) => {}
                Err(DomainError::NotFound { .. }) => {
                    self.book_repository.create(&user_id, &book).await?;
                }
                Err(e) => return Err(e.into()),
            }
            Ok(Some(BookDto::from(book)))
        }
        HistoryOperation::Delete => {
            match self.book_repository.delete(&user_id, &event.book_id).await {
                Ok(()) | Err(DomainError::NotFound { .. }) => {}
                Err(e) => return Err(e.into()),
            }
            Ok(None)
        }
    }

`RestoreAuthorInteractor::restore` — same structure. For `Create`/`Update`,
call `author_repository.update`; if `NotFound`, call `author_repository.create`
(this is the fallback that was missing in Phase 1). For `Delete`, call
`author_repository.delete`, treating `NotFound` as success.

The missing create fallback for author restore was identified in a code review
(CodeRabbit comment). Previously `RestoreAuthorInteractor` would fail with
`NotFound` when attempting to restore an author that had since been deleted.

**`src/use_case/traits/mutation.rs`**

Update `MutationUseCase`:
- `restore_book` returns `Result<Option<BookDto>, UseCaseError>`
- `restore_author` returns `Result<Option<AuthorDto>, UseCaseError>`

**`src/use_case/interactor/mutation.rs`**

Update `MutationInteractor::restore_book` and `restore_author` delegation
to propagate the `Option` return.

Verify: `cargo build && cargo test` must pass.

### Milestone 13 — Refactor Presentation — Nullable GraphQL Fields

**`src/presentation/graphql/object.rs`**

Rename `BookHistoryEntry` → `BookEventEntry`, `AuthorHistoryEntry` →
`AuthorEventEntry`.

Data fields that are nullable for delete events must be `Option<T>`:

    #[derive(SimpleObject)]
    pub struct BookEventEntry {
        pub event_id: ID,
        pub event_set_id: ID,
        pub operation: String,
        pub book_id: ID,
        pub title: Option<String>,
        pub author_ids: Vec<ID>,          // empty for delete
        pub isbn: Option<String>,
        pub read: Option<bool>,
        pub owned: Option<bool>,
        pub priority: Option<i32>,
        pub format: Option<BookFormat>,
        pub store: Option<BookStore>,
        pub book_created_at: Option<i64>, // unix timestamp; None for delete
        pub book_updated_at: Option<i64>,
        pub changed_at: i64,
    }

    #[derive(SimpleObject)]
    pub struct AuthorEventEntry {
        pub event_id: ID,
        pub event_set_id: ID,
        pub operation: String,
        pub author_id: ID,
        pub name: Option<String>,
        pub yomi: Option<String>,
        pub author_created_at: Option<i64>,
        pub author_updated_at: Option<i64>,
        pub changed_at: i64,
    }

Update `From<BookEventDto>` and `From<AuthorEventDto>` impls.

**`src/presentation/graphql/query.rs`**

Update the `bookHistory` / `authorHistory` query resolvers to use the renamed
interactors and return `Vec<BookEventEntry>` / `Vec<AuthorEventEntry>`.

**`src/presentation/graphql/mutation.rs`**

Update `restore_book` to return `Result<Option<Book>, PresentationalError>` and
`restore_author` to return `Result<Option<Author>, PresentationalError>`. Map
`None` from the use case to `Ok(None)`.

Regenerate `schema.graphql`:

    cargo run --bin gen_schema 2>/dev/null > schema.graphql

Verify: `cargo build && cargo test` must pass.

### Milestone 14 — Update All Tests

This milestone updates every test that references Phase 1 names or asserts
pre-state semantics, and adds new tests for the changed restore behavior.

**`src/infrastructure/book_history_repository.rs` (now `book_event_repository.rs`)**

Update the `find_by_book_returns_history_ordered_desc` test: after create then
update, the most recent entry should have `operation = Update` and
`title = "updated"` (post-update state), not `"original"`. Update the comment
from "pre-update snapshot" to "post-update state".

**`src/infrastructure/book_repository.rs` tests**

Update `test_update_records_history`: the history row for the update should
contain the *new* title, not the old one. Update assertions and comments
accordingly.

Update `test_delete_records_history`: the delete event row should have
`operation = "delete"` and all data fields `None` / absent. No
`book_event_author` rows should exist for delete events.

**`src/use_case/interactor/history.rs` unit tests**

Add `restore_book_delete_event_deletes_book` — mock `find_by_event_id` to
return an event with `operation = Delete`; assert `book_repository.delete`
is called and `restore` returns `Ok(None)`.

Add `restore_author_falls_back_to_create_when_deleted` — mock
`find_by_event_id` to return a `Create`/`Update` event; mock
`author_repository.update` to return `NotFound`; assert
`author_repository.create` is called and result is `Ok(Some(...))`.

Add `restore_author_delete_event_deletes_author` — analogous to the book
version.

**`src/use_case/interactor/mutation.rs` delegation tests**

Improve `restore_book_delegates_to_sub_use_case` and
`restore_author_delegates_to_sub_use_case` to assert the exact arguments
passed to the sub-use-case (using `eq("user1")` and `eq(42)` predicates) and
add a separate case that stubs the sub-use-case to return `Err` and verifies
the error propagates unchanged.

**E2E tests**

Add an E2E test case: create a book, delete it, call `restoreBook` with the
delete event's ID, verify the GraphQL response is `null` and the book is
absent from `books`.

Add an E2E test case: create a book, delete it, call `restoreBook` with the
create event's ID (restoring to the post-create state), verify the book is
recreated.

Run `cargo fmt --check && cargo clippy --fix --all-targets -- -D warnings &&
cargo test` and ensure all pass before committing.

---

## Phase 3 — Restore/Snapshot Operations and Extra Metadata

### Overview

Phase 2 built a post-state event log with three operation types: `create`,
`update`, `delete`. Phase 3 extends this with:

1. **`restore` operation** — records the fact that a restore was performed,
   what state it produced, and which event it was restored from.
2. **`snapshot` operation** — a migration-time checkpoint of every existing
   entity's current state, giving the event log a baseline for entities that
   predate Phase 1.
3. **`extra jsonb` column** — a flexible field on `book_event` and
   `author_event` for operation-specific data that does not warrant a dedicated
   column. Populated only for operations that need it (currently only
   `restore`). Includes a `version` key to enable future schema evolution.

For `restore` events, `extra` contains:
```json
{"version": 1, "source_event_id": <i64>}
```

For all other operations `extra` is `NULL`.

The `restore` operation is recorded *inside the repository*, in the same
transaction as the underlying upsert/delete, keeping the same pattern used for
`create`, `update`, and `delete`. A new `restore` method is added to
`BookRepository` and `AuthorRepository`; the interactors call `restore`
instead of `update`/`create`/`delete` directly.

The snapshot migration creates one `event_set` per user (operation =
`snapshot`), with all of that user's books and authors inserted as individual
event rows referencing that `event_set`. This preserves the invariant that a
single `event_set` groups a cohesive set of events.

### Progress

**Phase 3 — Restore/Snapshot + extra** (started 2026-04-30)

- [x] Milestone 15: Migration — rename history_operation, add ops, extra column, insert snapshots
  - [x] plan updated
- [x] Milestone 16: Domain — new HistoryOperation variants, extra field, restore method on repository traits
  - [x] plan updated
- [x] Milestone 17: Infrastructure — implement restore in Pg repositories, update event row structs
  - [x] plan updated
- [x] Milestone 18: Use case — update DTOs, update restore interactors
  - [x] plan updated
- [x] Milestone 19: Presentation — extra field in GraphQL types, regenerate schema
  - [x] plan updated
- [x] Milestone 20: Tests — unit and E2E
  - [x] plan updated
- [x] Milestone 21: Documentation — docs/database.md
  - [x] plan updated

### Decision Log (Phase 3)

- Decision: Column name for operation-specific JSONB is `extra`, not
  `metadata`. `metadata` implies "data about data" (too broad); `extra`
  honestly conveys "additional fields that don't warrant a dedicated column".
  Date/Author: 2026-04-30 / hiterm

- Decision: `restore` is recorded as a new event inside `BookRepository::restore`
  / `AuthorRepository::restore` (same transaction pattern as create/update/delete).
  The interactors call `repository.restore(source_event_id, Option<&Entity>)`
  instead of `update/create/delete` directly. This keeps event recording in the
  infrastructure layer and gives `restore` its own operation type.
  Date/Author: 2026-04-30 / hiterm

- Decision: Snapshot migration creates one `event_set` per user, grouping all of
  that user's books and authors in a single logical "snapshot" operation. This
  matches the intended semantics of `event_set` as a grouping of events belonging
  to one user action.
  Date/Author: 2026-04-30 / hiterm

- Decision: When restoring a `restore` or `snapshot` event, treat it the same
  as `create`/`update` — apply the stored state. Both `restore` and `snapshot`
  record a non-null post-state, so the same upsert path applies.
  Date/Author: 2026-04-30 / hiterm

### Milestone 15 — Migration

Create a new migration file. Use `date +%Y%m%d%H%M%S` to generate the prefix.

```sql
-- Rename history_operation → event_operation
ALTER TABLE history_operation RENAME TO event_operation;

-- Add restore and snapshot operation types
INSERT INTO event_operation VALUES ('restore'), ('snapshot');

-- Add restore_book, restore_author, snapshot to event_set_operation
INSERT INTO event_set_operation VALUES
  ('restore_book'), ('restore_author'), ('snapshot');

-- Add extra column to event tables
ALTER TABLE book_event ADD COLUMN extra jsonb;
ALTER TABLE author_event ADD COLUMN extra jsonb;

-- Insert snapshot events for all existing entities (one event_set per user)
WITH user_ids AS (
  SELECT DISTINCT user_id FROM book
  UNION
  SELECT DISTINCT user_id FROM author
),
new_sets AS (
  INSERT INTO event_set (id, user_id, operation)
  SELECT gen_random_uuid(), user_id, 'snapshot'
  FROM user_ids
  RETURNING id, user_id
),
new_book_events AS (
  INSERT INTO book_event
    (event_set_id, operation, book_id, user_id,
     title, isbn, read, owned, priority, format, store,
     book_created_at, book_updated_at)
  SELECT
    ns.id, 'snapshot', b.id, b.user_id,
    b.title, b.isbn, b.read, b.owned, b.priority, b.format, b.store,
    b.created_at, b.updated_at
  FROM book b
  JOIN new_sets ns ON b.user_id = ns.user_id
  RETURNING event_id, book_id
),
_book_event_authors AS (
  INSERT INTO book_event_author (event_id, author_id)
  SELECT nbe.event_id, ba.author_id
  FROM new_book_events nbe
  JOIN book_author ba ON ba.book_id = nbe.book_id
)
INSERT INTO author_event
  (event_set_id, operation, author_id, user_id,
   name, yomi, author_created_at, author_updated_at)
SELECT
  ns.id, 'snapshot', a.id, a.user_id,
  a.name, a.yomi, a.created_at, a.updated_at
FROM author a
JOIN new_sets ns ON a.user_id = ns.user_id;
```

Verify: `cargo test` must pass (sqlx::test creates fresh DB from migrations).

### Milestone 16 — Domain Layer

**`src/domain/entity/history.rs`**

Add `Restore` and `Snapshot` variants to `HistoryOperation`. Update
`as_str` and `TryFrom<&str>`.

Add `extra: Option<serde_json::Value>` to `BookEvent` and `AuthorEvent`.

**`src/domain/repository/book_repository.rs`**

Add `restore` to `BookRepository`:

```rust
async fn restore(
    &self,
    user_id: &UserId,
    source_event_id: i64,
    book: Option<&Book>,   // None = entity should be deleted
) -> Result<(), DomainError>;
```

**`src/domain/repository/author_repository.rs`**

Add `restore` to `AuthorRepository`:

```rust
async fn restore(
    &self,
    user_id: &UserId,
    source_event_id: i64,
    author: Option<&Author>,
) -> Result<(), DomainError>;
```

Verify: `cargo build` must succeed.

### Milestone 17 — Infrastructure Layer

**`Cargo.toml`**: add `"json"` to sqlx features.

**`src/infrastructure/book_event_repository.rs`**

Add `extra: Option<serde_json::Value>` to `BookEventRow`. Update
`row_to_book_event` to pass it through. Update both SELECT queries to
include `be.extra`.

**`src/infrastructure/author_event_repository.rs`**

Same pattern for `AuthorEventRow`.

**`src/infrastructure/book_repository.rs`**

Implement `BookRepository::restore`:

```
BEGIN;
  if book is Some(b):
    -- Try UPDATE; if 0 rows affected, INSERT instead
    -- INSERT INTO event_set (operation='restore_book')
    -- INSERT INTO book_event (operation='restore', extra='{"version":1,"source_event_id":<id>}', all data fields)
    -- INSERT INTO book_event_author for each author_id
  else (book is None):
    -- DELETE FROM book_author WHERE user_id=$1 AND book_id=$2 (ignore NotFound)
    -- DELETE FROM book WHERE user_id=$1 AND id=$2 (ignore NotFound)
    -- INSERT INTO event_set (operation='restore_book')
    -- INSERT INTO book_event (operation='restore', extra='{"version":1,"source_event_id":<id>}', data fields NULL)
COMMIT;
```

**`src/infrastructure/author_repository.rs`**

Implement `AuthorRepository::restore` using the same pattern.

Verify: `cargo build && cargo test` must pass.

### Milestone 18 — Use Case Layer

**`src/use_case/dto/history.rs`**

Add `extra: Option<serde_json::Value>` to `BookEventDto` and `AuthorEventDto`.
Update `From<BookEvent>` and `From<AuthorEvent>`.

**`src/use_case/interactor/history.rs`**

Update `RestoreBookInteractor::restore`: call
`book_repository.restore(&user_id, event_id, Option<&Book>)` instead of
`update`/`create`/`delete`. Match on `Restore` and `Snapshot` like `Create`
and `Update`.

Update `RestoreAuthorInteractor::restore` the same way.

Verify: `cargo test` must pass.

### Milestone 19 — Presentation Layer

**`src/presentation/graphql/object.rs`**

Add `extra: Option<Json<serde_json::Value>>` to `BookEventEntry` and
`AuthorEventEntry`. Update `From` impls.

Regenerate `schema.graphql`:

```
cargo run --bin gen_schema 2>/dev/null > schema.graphql
```

### Milestone 20 — Tests

Unit tests to add/update in `src/use_case/interactor/history.rs`:

- Update `restore_book_success`: expect `book_repo.restore()` instead of
  `book_repo.update()`.
- Update `restore_book_falls_back_to_create_when_deleted`: no longer needed
  since `repository.restore` handles the upsert internally. Remove or replace
  with `restore_book_success_calls_restore_with_source_event_id`.
- Update `restore_book_delete_event_deletes_book`: expect `book_repo.restore()`
  with `None` argument.
- Same updates for author variants.
- Add `restore_book_snapshot_event_applies_state` and
  `restore_author_snapshot_event_applies_state`.

E2E tests to add in `e2e/tests/e2e.rs`:

- `e2e_restore_book_records_restore_event`: restore a book, then call
  `bookHistory`; verify the most recent entry has `operation = "restore"` and
  `extra` contains `source_event_id`.
- `e2e_restore_author_records_restore_event`: same for author.
- `e2e_snapshot_events_exist_for_all_entities`: not applicable for a fresh test
  DB (snapshot migration runs on existing data only). Skip or note in comment.

### Milestone 21 — Documentation

Create `docs/database.md` documenting:

- Overview of the event log design
- Table descriptions: `event_set`, `event_set_operation`, `event_operation`,
  `book_event`, `book_event_author`, `author_event`
- `extra` field schema by operation type:
  - `restore`: `{"version": 1, "source_event_id": <i64>}`
  - all other operations: `null`
- Version history for the `extra` schema

---

## Phase 4 — Cleanup and Consistency

### Overview

Post-Phase 3 cleanup addressing naming inconsistencies, migration hygiene, and
test coverage gaps identified during review.

1. **`HistoryOperation` → `EventOperation`** — the DB table was renamed to
   `event_operation` in Phase 3, but the Rust enum kept the old name. Rename
   the enum and all references for consistency.
2. **`snapshot` → `snapshot_all` in `event_set_operation`** — distinguish the
   bulk migration snapshot (all entities for a user) from a potential future
   per-entity snapshot operation.
3. **Consolidate PR migrations into one** — the PR introduced two files
   (`20260429040611_add_change_history.sql` and `20260430102404_add_restore_snapshot.sql`);
   neither has been applied to production, so they can be merged into one clean file.
4. **Migration data test** — verify that the snapshot CTE in the migration
   produces the correct rows; still under discussion.

### Progress

**Phase 4 — Cleanup** (started 2026-04-30)

- [x] Milestone 22: Rename `HistoryOperation` → `EventOperation` in all Rust files
  - [x] plan updated
- [x] Milestone 23: `snapshot_all` rename + consolidate migrations to one file
  - [x] plan updated
- [x] Milestone 24: Migration data test
  - [x] plan updated

### Decision Log (Phase 4)

- Decision: `HistoryOperation` → `EventOperation`. The DB lookup table was already
  renamed from `history_operation` to `event_operation` in Phase 3; the Rust type
  should mirror it. Module names (`src/domain/entity/history.rs`, `pub mod history`)
  are kept as-is because they refer to the change-history feature, not the DB table.
  Date/Author: 2026-04-30 / hiterm

- Decision: `event_set_operation.snapshot` → `snapshot_all`. The event_set-level
  operation describes a bulk snapshot of all entities for a user, which `snapshot_all`
  names more precisely. The per-entity event_operation value (`snapshot`) remains
  unchanged because each row is always a single-entity snapshot regardless of trigger.
  This leaves room for a future per-entity `snapshot` event_set operation if needed.
  Date/Author: 2026-04-30 / hiterm

- Decision: Consolidate the two PR migration files into one
  (`20260429040611_add_change_history.sql`). Both were introduced in this PR and have
  never been applied to production, so merging carries no migration risk. A single file
  is simpler: it creates `event_operation` directly (no `history_operation` → rename),
  includes `restore`/`snapshot` from the start, includes `extra jsonb` columns, and
  runs the snapshot CTE.
  Date/Author: 2026-04-30 / hiterm

- Decision: Migration test is a Node.js script (`migrations/test/test_migration.mjs`)
  using `child_process.execSync` to call `psql` — no extra npm dependencies. CI creates
  two dedicated databases (empty and data scenarios) so the script can apply migrations
  from a clean state in each case. Tests verify both counts and field-level content of
  snapshot rows, plus the empty-DB edge case.
  Date/Author: 2026-04-30 / hiterm

---

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
       cargo clippy --fix --all-targets -- -D warnings
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
