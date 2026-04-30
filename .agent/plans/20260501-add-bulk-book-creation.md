# Add Import Books Mutation

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up
to date as work proceeds.

This document must be maintained in accordance with `.agent/PLANS.md`.

**Plan update rule**: Update this document continuously as work proceeds —
mark each task done the moment it is completed, record discoveries immediately
when found, and log decisions as soon as they are made. Do NOT batch updates
and apply them all at the end.

**Commit granularity rule**: Commit at each logical breakpoint — completing a
migration file, adding a new domain trait, implementing a repository method,
adding a test suite, and so on. Do not batch unrelated changes into one
commit. Each commit message must describe what specifically changed and why.


## Purpose / Big Picture

Before this change, registering multiple books requires issuing N separate
`createBook` GraphQL mutations, each in its own HTTP round-trip and its own
database transaction. Clients that import large reading lists must sequence
every request and handle partial failures themselves.

After this change, clients can import many books in a single `importBooks`
GraphQL mutation. All books and any author records that need to be created
are inserted inside **one database transaction**, and the entire batch is
recorded under a **single `event_set`** in the event log so that the whole
import is traceable as one logical operation.

This mutation is designed for a specific use case: bringing in a list of books
from an external source (such as a reading-list export or a CSV file) where
authors are identified by name, not by pre-existing IDs. Author names are
supplied per book as strings. The server resolves each name via a
**find-or-create** strategy: if an author with that name already exists for
the authenticated user it is reused; otherwise a new author row is inserted.
A `UNIQUE(user_id, name)` constraint on the `author` table enforces this
invariant at the database level.

To see it working after implementation: call `importBooks` with two or more
books whose author lists overlap, then query `bookHistory` for each returned
book. All events should share the same `eventSetId`, confirming they were
recorded as a single batch.


## Progress

- [ ] Milestone 1: Database migrations — unique constraint + new event_set_operation value
  - [ ] plan updated
- [ ] Milestone 2: Domain layer — ImportBookInput struct + ImportBooksRepository trait
  - [ ] plan updated
- [ ] Milestone 3: Infrastructure layer — PgImportBooksRepository (find-or-create + single tx)
  - [ ] plan updated
- [ ] Milestone 4: Use case layer — ImportBookEntryDto, ImportBooksUseCase, ImportBooksInteractor,
  update MutationInteractor
  - [ ] plan updated
- [ ] Milestone 5: Presentation layer and DI — GraphQL input/resolver, wire everything up
  - [ ] plan updated
- [ ] Milestone 6: Tests — unit tests and E2E tests
  - [ ] plan updated


## Surprises & Discoveries

*(Record unexpected behaviors, bugs, or insights here as they occur.)*


## Decision Log

- Decision: The mutation is named `importBooks` rather than `createBulkBooks`
  or a generic "bulk create" variant.
  Rationale: This mutation embeds specific semantics — author names instead
  of IDs, find-or-create author resolution, a single event_set for the whole
  batch — that make it a purpose-built import operation, not a generic
  multi-item create. The name `importBooks` communicates that intent clearly
  to API clients.
  Date/Author: 2026-05-01 / hiterm

- Decision: Author input uses `authorNames: [String!]!` (name strings) rather
  than `authorIds`. The server resolves names to IDs via find-or-create in the
  infrastructure layer.
  Rationale: Clients doing an import typically have author names from an
  external source, not pre-existing author IDs. Requiring IDs would force the
  client to do N `createAuthor` calls before the import, defeating the purpose.
  Date/Author: 2026-05-01 / hiterm

- Decision: A single `event_set` row (operation = `import_books`) covers all
  author events and book events produced by one `importBooks` call.
  Rationale: The `event_set` table was designed to group related events from
  one logical user action. An import is exactly such an action; grouping
  everything under one `event_set` makes it queryable as a unit.
  Date/Author: 2026-05-01 / hiterm

- Decision: `ImportBooksRepository` is a separate domain trait, not an
  extension of `BookRepository`. The infrastructure implementation is in a
  new file `src/infrastructure/import_books_repository.rs`.
  Rationale: The import method touches both the `book` and `author` tables in
  a single transaction. Hanging it off `BookRepository` would couple that
  trait to author-management concerns. A dedicated trait keeps
  responsibilities clean.
  Date/Author: 2026-05-01 / hiterm

- Decision: Newly-inserted-author detection uses `rows_affected()` from the
  `ON CONFLICT DO NOTHING` INSERT, not a comparison of candidate vs returned
  ID.
  Rationale: `rows_affected()` returns 1 for a new insert and 0 for a
  conflict path; this is unambiguous and does not require carrying the
  candidate UUID through a subsequent SELECT.
  Date/Author: 2026-05-01 / hiterm


## Outcomes & Retrospective

*(Fill in when all milestones are complete.)*


## Context and Orientation

This repository is a Rust/async-graphql API backed by PostgreSQL (via sqlx).
It follows a strict four-layer architecture; understanding it is essential
before touching any file.

**Domain layer** (`src/domain/`): Pure Rust entities and repository traits.
No database access. Repository traits carry `#[automock]` (from the
`mockall` crate) so unit tests can inject mock implementations without a real
database. New domain traits go in `src/domain/repository/<name>.rs`, and
their module is declared in `src/domain/repository.rs`.

**Infrastructure layer** (`src/infrastructure/`): Concrete `Pg*Repository`
structs that own a `sqlx::Pool<Postgres>` and implement the domain traits
using SQL queries. Event recording belongs here — domain traits and use-case
interactors must never be aware of it. New implementations go in
`src/infrastructure/<name>.rs`, declared in `src/infrastructure.rs`.

**Use case layer** (`src/use_case/`): Interactors contain business logic and
depend only on domain repository traits. DTOs (plain Rust structs with public
fields) carry data between layers. Traits for each use-case operation live in
`src/use_case/traits/<topic>.rs` (declared in `src/use_case/traits.rs`).
Interactors live in `src/use_case/interactor/<topic>.rs` (declared in
`src/use_case/interactor.rs`).

The **`MutationInteractor`** struct in `src/use_case/interactor/mutation.rs`
is a façade: it holds one field per mutation use-case and delegates every
`MutationUseCase` method to the appropriate field. It currently has nine
generic type parameters (one per sub-use-case). Adding a new mutation
requires appending a tenth parameter and a matching field.

**Presentation layer** (`src/presentation/graphql/`): async-graphql schema,
resolvers, and GraphQL input/output types. The mutation resolver lives in
`src/presentation/graphql/mutation.rs`. GraphQL input types and output objects
live in `src/presentation/graphql/object.rs`.

**Dependency injection** (`src/dependency_injection.rs`): assembles
everything for production use. `MI` is the concrete type alias for
`MutationInteractor` with all production type params filled in.

Key files relevant to this plan:

    src/domain/repository.rs                          — module declarations
    src/domain/repository/book_repository.rs          — BookRepository trait
    src/domain/repository/author_repository.rs        — AuthorRepository trait
    src/domain/entity/author.rs                       — Author, AuthorId, AuthorName
    src/domain/entity/book.rs                         — Book, BookId, BookTitle, …
    src/infrastructure.rs                             — module declarations
    src/infrastructure/book_repository.rs             — PgBookRepository
    src/infrastructure/author_repository.rs           — PgAuthorRepository
    src/use_case/traits.rs                            — module declarations
    src/use_case/traits/book.rs                       — CreateBookUseCase, …
    src/use_case/traits/mutation.rs                   — MutationUseCase trait
    src/use_case/interactor.rs                        — module declarations
    src/use_case/interactor/book.rs                   — CreateBookInteractor, …
    src/use_case/interactor/mutation.rs               — MutationInteractor + tests
    src/use_case/dto/book.rs                          — BookDto, CreateBookDto, …
    src/presentation/graphql/object.rs                — GraphQL types
    src/presentation/graphql/mutation.rs              — mutation resolvers
    src/dependency_injection.rs                       — wiring + MI/QI type aliases
    migrations/20220306122339_create_tables.sql       — base schema
    migrations/20260429040611_add_event_tables.sql    — event log schema
    e2e/tests/e2e.rs                                  — E2E test suite

**Existing event log schema** (relevant tables):

    event_set            — one row per logical user action; references event_set_operation
    event_set_operation  — lookup table of allowed operation strings
    event_operation      — lookup table: 'create', 'update', 'delete', 'restore', 'snapshot'
    book_event           — one row per book-related event; references event_set
    book_event_author    — join table between book_event and authors
    author_event         — one row per author-related event; references event_set

The `event_set_operation` table currently contains: `create_book`,
`update_book`, `delete_book`, `create_author`, `update_author`,
`delete_author`, `restore_book`, `restore_author`, `snapshot_all`. We need to
add `import_books`.

**Existing `author` table** (from `migrations/20220306122339_create_tables.sql`):

    CREATE TABLE author (
      id         uuid    NOT NULL,
      user_id    text    NOT NULL,
      name       text    NOT NULL,
      yomi       text    NOT NULL DEFAULT '',
      created_at timestamptz NOT NULL DEFAULT current_timestamp,
      updated_at timestamptz NOT NULL DEFAULT current_timestamp,
      PRIMARY KEY (id, user_id),
      FOREIGN KEY (user_id) REFERENCES bookshelf_user(id)
    );

There is currently no unique constraint on `(user_id, name)`. Milestone 1
adds one so find-or-create can rely on `ON CONFLICT (user_id, name) DO
NOTHING`.

**Terminology used in this plan:**

- *find-or-create*: attempt to insert a new row; if a uniqueness conflict
  occurs, silently do nothing and then select the existing row. The caller
  gets back the row's ID regardless of whether a new row was created.
- *`event_set`*: a single row that groups all the events produced by one
  logical user action (in this case, one `importBooks` call).
- *`event_set_operation`*: the operation label on the `event_set` row (e.g.
  `import_books`); different from the per-event `operation` column (e.g.
  `create`, `update`).


## Plan of Work

### Milestone 1 — Database Migrations

Two SQL migration files are needed, applied in timestamp order. Generate each
timestamp with `date +%Y%m%d%H%M%S` at the time of creation so sqlx orders
them correctly.

The first migration adds a `UNIQUE(user_id, name)` constraint to the
`author` table. Without this constraint the `ON CONFLICT` clause in the
find-or-create logic will not compile at the database level.

    -- migrations/<ts>_add_author_name_unique.sql
    ALTER TABLE author
      ADD CONSTRAINT author_user_id_name_unique UNIQUE (user_id, name);

**Important**: existing data must not contain duplicate `(user_id, name)`
pairs for this migration to succeed. On a fresh development or CI database
there are no duplicates. If running against a database with pre-existing data,
deduplicate first (this is documented in the Idempotence section below).

The second migration inserts the new operation label into the lookup table:

    -- migrations/<ts>_add_import_books_operation.sql
    INSERT INTO event_set_operation VALUES ('import_books');

After writing both files, verify they apply cleanly by running
`sqlx migrate run` (requires `DATABASE_URL` to be set). Alternatively,
`cargo test` triggers sqlx to create a fresh test database from all migrations
and will fail if either migration has a syntax error.

### Milestone 2 — Domain Layer

Create `src/domain/repository/import_books_repository.rs`. This file defines
the input struct and the trait that the use-case layer will depend on.

`ImportBookInput` is a plain struct with no database concerns:

    use async_trait::async_trait;
    use mockall::automock;
    use time::OffsetDateTime;

    use crate::common::types::{BookFormat, BookStore};
    use crate::domain::entity::{
        author::AuthorName,
        book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
        user::UserId,
    };
    use crate::domain::entity::book::Book;
    use crate::domain::error::DomainError;

    #[derive(Clone)]
    pub struct ImportBookInput {
        pub book_id: BookId,
        pub title: BookTitle,
        pub author_names: Vec<AuthorName>,
        pub isbn: Isbn,
        pub read: ReadFlag,
        pub owned: OwnedFlag,
        pub priority: Priority,
        pub format: BookFormat,
        pub store: BookStore,
        pub created_at: OffsetDateTime,
        pub updated_at: OffsetDateTime,
    }

    #[automock]
    #[async_trait]
    pub trait ImportBooksRepository: Send + Sync + 'static {
        async fn import(
            &self,
            user_id: &UserId,
            books: Vec<ImportBookInput>,
        ) -> Result<Vec<Book>, DomainError>;
    }

The `#[automock]` attribute causes `mockall` to generate a
`MockImportBooksRepository` struct used by the unit tests in Milestone 6.

Add `pub mod import_books_repository;` to `src/domain/repository.rs`.

After this milestone `cargo build` must succeed with no warnings.

### Milestone 3 — Infrastructure Layer

Create `src/infrastructure/import_books_repository.rs`. This file contains
`PgImportBooksRepository`, which owns a `PgPool` and implements
`ImportBooksRepository`.

The entire `import` method runs inside a single PostgreSQL transaction
(`pool.begin().await?` … `tx.commit().await?`). The steps are:

**Step 1 — generate the shared event_set ID.**

Before touching any rows, call `Uuid::new_v4()` once and bind it to
`es_id`. This UUID will be reused for the single `event_set` row, all
`author_event` rows, and all `book_event` rows. Insert the `event_set` row
immediately so the foreign-key references in later inserts are satisfied:

    INSERT INTO event_set (id, user_id, operation)
    VALUES ($1, $2, 'import_books')

**Step 2 — collect unique author names and build the name-to-ID map.**

Iterate over every `ImportBookInput.author_names` (across all books) and
de-duplicate them using a `HashMap<String, AuthorId>`. For each unique name:

a. Call `Uuid::new_v4()` to produce a candidate ID.

b. Execute the find-or-create INSERT:

    INSERT INTO author (id, user_id, name)
    VALUES ($candidate_id, $user_id, $name)
    ON CONFLICT (user_id, name) DO NOTHING

c. Check `rows_affected()` from the INSERT result. If it is 1, a new author
   was created; if it is 0, the name already existed.

d. Fetch the authoritative row (works for both new and existing rows because
   the query runs inside the same transaction):

    SELECT id, yomi, created_at, updated_at
    FROM author
    WHERE user_id = $user_id AND name = $name

e. Record the returned `id` in the name-to-ID map.

f. If `rows_affected()` was 1 (new author inserted), also insert an
   `author_event` row referencing `es_id`:

    INSERT INTO author_event
      (event_set_id, operation, author_id, user_id,
       name, yomi, author_created_at, author_updated_at)
    VALUES ($es_id, 'create', $returned_id, $user_id,
            $name, $yomi, $created_at, $updated_at)

   Note that `yomi`, `created_at`, and `updated_at` come from the SELECT in
   step 2d, not from the application — the DB fills in defaults and we fetch
   them back.

**Step 3 — insert books and book events.**

For each `ImportBookInput` (in the original order):

a. Resolve author IDs: look up each `author_name` in the map built in step 2.

b. Build a `Book` entity via `Book::new(...)` using the resolved author IDs.

c. Insert the book row:

    INSERT INTO book
      (id, user_id, title, isbn, read, owned, priority, format, store,
       created_at, updated_at)
    VALUES (...)

d. If the book has authors, insert `book_author` rows:

    INSERT INTO book_author (user_id, book_id, author_id)
    SELECT $user_id, $book_id, unnest($author_ids::uuid[])

e. Insert a `book_event` row referencing `es_id`:

    INSERT INTO book_event
      (event_set_id, operation, book_id, user_id,
       title, isbn, read, owned, priority, format, store,
       book_created_at, book_updated_at)
    VALUES ($es_id, 'create', ...)
    RETURNING event_id

f. If the book has authors, insert `book_event_author` rows:

    INSERT INTO book_event_author (event_id, author_id)
    SELECT $event_id, unnest($author_ids::uuid[])

g. Push the `Book` entity into the result vector.

**Step 4 — commit.**

    tx.commit().await?

Return the result vector. Any error anywhere rolls back the entire transaction
automatically (sqlx drops the transaction on error without committing).

Add `pub mod import_books_repository;` to `src/infrastructure.rs`.

After this milestone `cargo build && cargo test` must pass.

### Milestone 4 — Use Case Layer

There are four changes in this milestone: add a DTO, add a use-case trait,
add an interactor, and update `MutationInteractor`.

**Add `ImportBookEntryDto` to `src/use_case/dto/book.rs`.**

    #[derive(Debug, Clone)]
    pub struct ImportBookEntryDto {
        pub title: String,
        pub author_names: Vec<String>,
        pub isbn: String,
        pub read: bool,
        pub owned: bool,
        pub priority: i32,
        pub format: BookFormat,
        pub store: BookStore,
    }

**Add `ImportBooksUseCase` to `src/use_case/traits/book.rs`.**

    #[automock]
    #[async_trait]
    pub trait ImportBooksUseCase: Send + Sync + 'static {
        async fn import(
            &self,
            user_id: &str,
            books: Vec<ImportBookEntryDto>,
        ) -> Result<Vec<BookDto>, UseCaseError>;
    }

**Add `ImportBooksInteractor` to `src/use_case/interactor/book.rs`.**

The interactor holds an `IBR: ImportBooksRepository` and implements
`ImportBooksUseCase`:

    pub struct ImportBooksInteractor<IBR> {
        import_books_repository: IBR,
    }

    impl<IBR> ImportBooksInteractor<IBR> {
        pub fn new(import_books_repository: IBR) -> Self {
            Self { import_books_repository }
        }
    }

The `import` implementation:

1. Parse `user_id` into `UserId::new(user_id.to_string())?`.
2. Capture `now = OffsetDateTime::now_utc()` once.
3. Map each `ImportBookEntryDto` into an `ImportBookInput`, validating all
   fields (title, isbn, author names, priority). If any validation fails,
   return a `UseCaseError` immediately without calling the repository.
4. Call `self.import_books_repository.import(&user_id, inputs).await?`.
5. Map each returned `Book` to a `BookDto` and return the vector.

The imports needed in `book.rs` for this interactor:

    use crate::domain::entity::author::AuthorName;
    use crate::domain::repository::import_books_repository::{
        ImportBookInput, ImportBooksRepository,
    };
    use crate::use_case::dto::book::ImportBookEntryDto;
    use crate::use_case::traits::book::ImportBooksUseCase;

**Update `MutationUseCase` in `src/use_case/traits/mutation.rs`.**

Add a method:

    async fn import_books(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<Vec<BookDto>, UseCaseError>;

This requires importing `ImportBookEntryDto` at the top of the file.

**Update `MutationInteractor` in `src/use_case/interactor/mutation.rs`.**

Add a tenth generic parameter `IBUC` (for `ImportBooksUseCase`) appended
after the existing nine parameters. Add a field
`import_books_use_case: IBUC`. Update `MutationInteractor::new` to accept it
as the tenth argument. Keep the existing
`#[allow(clippy::too_many_arguments)]` annotation on `new`.

The `where` clause on the `MutationUseCase` impl block gains:
`IBUC: ImportBooksUseCase`.

Implement `MutationUseCase::import_books` as a simple delegation:

    async fn import_books(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<Vec<BookDto>, UseCaseError> {
        self.import_books_use_case.import(user_id, books).await
    }

**Update the `#[cfg(test)]` block in `mutation.rs`.**

The `DefaultInteractor` type alias must gain a tenth type parameter
`MockImportBooksUseCase`. The `InteractorBuilder` struct gains a new field
`import_books: MockImportBooksUseCase` with a matching `with_import_books`
setter. The `build()` method passes the new field as the tenth argument to
`MutationInteractor::new`.

After this milestone `cargo test` must pass.

### Milestone 5 — Presentation Layer and Dependency Injection

**Add `ImportBookInput` to `src/presentation/graphql/object.rs`.**

    #[derive(InputObject)]
    pub struct ImportBookInput {
        pub title: String,
        pub author_names: Vec<String>,
        pub isbn: String,
        pub read: bool,
        pub owned: bool,
        pub priority: i32,
        pub format: BookFormat,
        pub store: BookStore,
    }

    impl From<ImportBookInput> for ImportBookEntryDto {
        fn from(input: ImportBookInput) -> Self {
            ImportBookEntryDto {
                title: input.title,
                author_names: input.author_names,
                isbn: input.isbn,
                read: input.read,
                owned: input.owned,
                priority: input.priority,
                format: input.format.into(),
                store: input.store.into(),
            }
        }
    }

This requires importing `ImportBookEntryDto` at the top of `object.rs`.

**Add a resolver to `src/presentation/graphql/mutation.rs`.**

Inside the `#[Object] impl<MUC> Mutation<MUC>` block, add:

    async fn import_books(
        &self,
        ctx: &Context<'_>,
        books: Vec<ImportBookInput>,
    ) -> Result<Vec<Book>, PresentationalError> {
        let claims = get_claims(ctx)?;
        let books = self
            .mutation_use_case
            .import_books(
                &claims.sub,
                books.into_iter().map(Into::into).collect(),
            )
            .await?;
        Ok(books.into_iter().map(Book::from).collect())
    }

**Update `src/dependency_injection.rs`.**

Import the new types:

    use crate::infrastructure::import_books_repository::PgImportBooksRepository;
    use crate::use_case::interactor::book::ImportBooksInteractor;

Instantiate inside `dependency_injection`:

    let import_books_repository = PgImportBooksRepository::new(pool.clone());
    let import_books_use_case = ImportBooksInteractor::new(import_books_repository);

Update the `MI` type alias to include the tenth parameter:

    pub type MI = MutationInteractor<
        RegisterUserInteractor<PgUserRepository>,
        CreateBookInteractor<PgBookRepository>,
        UpdateBookInteractor<PgBookRepository>,
        DeleteBookInteractor<PgBookRepository>,
        CreateAuthorInteractor<PgAuthorRepository>,
        UpdateAuthorInteractor<PgAuthorRepository>,
        DeleteAuthorInteractor<PgAuthorRepository>,
        RestoreBookInteractor<PgBookRepository, PgBookEventRepository>,
        RestoreAuthorInteractor<PgAuthorRepository, PgAuthorEventRepository>,
        ImportBooksInteractor<PgImportBooksRepository>,
    >;

Pass `import_books_use_case` as the tenth argument to
`MutationInteractor::new(...)`.

After this milestone `cargo build` must succeed. Regenerate the GraphQL
schema if a `schema.graphql` file exists in the repo:

    cargo run --bin gen_schema 2>/dev/null > schema.graphql

### Milestone 6 — Tests

**Unit tests for `ImportBooksInteractor`** — add to the `#[cfg(test)]` module
at the bottom of `src/use_case/interactor/book.rs`. Each test follows the
Given / When / Then pattern used throughout the file.

- `import_books_empty_list` — pass an empty `books` vec; the mock repository
  should be called with an empty vec and return `Ok(vec![])`; the interactor
  should return `Ok(vec![])`.
- `import_books_with_author_names` — pass two books each with one author name;
  verify the mock is called once with the two converted inputs and that the
  returned `BookDto` vector has two entries.
- `import_books_propagates_repository_error` — mock returns
  `Err(DomainError::Unexpected(...))`, verify the interactor returns
  `Err(UseCaseError::...)`.
- `import_books_invalid_title_returns_error` — pass a book with an empty
  title string; verify the interactor returns an error without calling the
  repository (the mock expects zero calls).
- `import_books_invalid_isbn_returns_error` — pass a book with an invalid
  ISBN (e.g. `"1"`); same pattern.
- `import_books_invalid_author_name_returns_error` — pass a book with an
  empty author name string; same pattern.

**Unit test for `MutationInteractor`** — add to the `#[cfg(test)]` module in
`src/use_case/interactor/mutation.rs`:

- `import_books_delegates_to_sub_use_case` — mock `MockImportBooksUseCase`,
  expect `import` to be called with exact arguments `eq("user1")` and a
  matching books vec; stub it to return `Ok(vec![make_book_dto(&book_id)])`.
  Verify the interactor returns `Ok` with that dto.

**E2E tests** — add to `e2e/tests/e2e.rs`. All E2E tests must be annotated
with `#[serial]` because they share the same running server.

`e2e_import_books` scenario:

1. Generate a UUID for `user_id`, register the user, obtain a JWT.
2. Pre-create an author named `"Existing Author"` via `createAuthor`; capture
   the returned author ID.
3. Call `importBooks` with `books`:
   - Book 1: `title="Book One"`, `authorNames=["Existing Author"]`, other
     fields set to valid defaults.
   - Book 2: `title="Book Two"`, `authorNames=["New Author"]`, other fields
     set to valid defaults.
4. Assert the response contains exactly two books with titles `"Book One"` and
   `"Book Two"`, each with a distinct non-empty UUID.
5. Query `bookHistory` for each returned book ID. Each must have at least one
   event with `operation="create"`. Both events must share the same
   `eventSetId`.
6. Query the authors of Book Two and verify one of them has name `"New
   Author"`.
7. Verify `"Existing Author"` was not duplicated by querying all authors for
   the user and counting entries with that name; expect exactly 1.
8. Cleanup: delete both books and both authors.

`e2e_import_books_empty` scenario:

1. Register a fresh user.
2. Call `importBooks(books: [])`.
3. Assert the response is an empty array with no error.


## Concrete Steps

Run all commands from the repository root
(`/home/hiterm/ghq/github.com/hiterm/bookshelf-api`) unless noted otherwise.

1. Create the first migration file. Use the current timestamp as the prefix:

       TS=$(date +%Y%m%d%H%M%S)
       touch migrations/${TS}_add_author_name_unique.sql

   Write the `ALTER TABLE author ADD CONSTRAINT ...` SQL into that file.

2. Create the second migration file immediately after (ensure a later
   timestamp so sqlx applies them in order):

       sleep 1
       TS2=$(date +%Y%m%d%H%M%S)
       touch migrations/${TS2}_add_import_books_operation.sql

   Write the `INSERT INTO event_set_operation VALUES ('import_books')` SQL
   into that file.

3. Apply migrations (requires `DATABASE_URL`):

       sqlx migrate run

   Expected output: two lines of the form `Applying migrations/…` followed by
   `Applied N migration(s)`.

4. Implement Milestone 2 (domain layer). After each file, verify:

       cargo build 2>&1 | head -40

5. Implement Milestone 3 (infrastructure layer). Verify:

       cargo build && cargo test

6. Implement Milestone 4 (use case layer). Verify:

       cargo test

7. Implement Milestone 5 (presentation layer and DI). Verify:

       cargo build

   Then regenerate the schema if applicable:

       cargo run --bin gen_schema 2>/dev/null > schema.graphql

8. Run pre-commit checks (mandatory per CLAUDE.md):

       cargo fmt --check
       cargo clippy --all-targets -- -D warnings
       cargo test

   If `cargo fmt --check` reports diffs, run `cargo fmt` and re-check.
   Fix all clippy warnings before committing.

9. Implement Milestone 6 (tests). After adding unit tests:

       cargo test

   After adding E2E tests, run them against a local server (see the
   Validation section for the exact commands).

10. Commit at each logical breakpoint per CLAUDE.md conventions.


## Validation and Acceptance

**Unit test acceptance**: `cargo test` must report 0 failures. The six new
`ImportBooksInteractor` tests and the one new `MutationInteractor` delegation
test must exist and pass. Running `cargo test` from the repo root exercises
all unit tests.

**E2E acceptance**: both new E2E scenarios (`e2e_import_books` and
`e2e_import_books_empty`) must pass. Run the E2E suite against a running
server:

    # Start the server (in a separate terminal, or use the existing test
    # server if one is already running):
    cargo run

    # In another terminal, set the test server URL and run the E2E tests:
    cd e2e
    TEST_SERVER_URL=http://localhost:8080 cargo test

    # Or, using docker-compose if configured:
    docker compose -f docker-compose-test.yml up -d
    cd e2e && TEST_SERVER_URL=http://localhost:8080 cargo test
    docker compose -f docker-compose-test.yml down

Both scenarios must report `ok`.

**Manual smoke test** (optional, requires a running server with a valid JWT):

    curl -s -X POST http://localhost:8080/graphql \
      -H "Authorization: Bearer <token>" \
      -H "Content-Type: application/json" \
      -d '{"query":"mutation { importBooks(books: [
            {title:\"Imported Book\", authorNames:[\"Author A\"],
             isbn:\"\", read:false, owned:false, priority:50,
             format:Unknown, store:Unknown}
          ]) { id title } }"}'

    # Expected: {"data":{"importBooks":[{"id":"<uuid>","title":"Imported Book"}]}}


## Idempotence and Recovery

Migrations use `ALTER TABLE … ADD CONSTRAINT` and `INSERT INTO … VALUES`.
Running them a second time will fail (`constraint already exists` /
`duplicate key value`). sqlx tracks applied migrations in `_sqlx_migrations`
and will not re-apply them, so this is safe in practice.

If the unique-constraint migration fails because duplicate `(user_id, name)`
pairs exist in `author`, identify and remove duplicates first:

    -- Find duplicates:
    SELECT user_id, name, COUNT(*) FROM author
    GROUP BY user_id, name HAVING COUNT(*) > 1;

    -- Resolve by deleting duplicate rows (keep the oldest by created_at):
    DELETE FROM author a
    USING (
        SELECT id, ROW_NUMBER() OVER (
            PARTITION BY user_id, name ORDER BY created_at
        ) AS rn FROM author
    ) dup
    WHERE a.id = dup.id AND dup.rn > 1;

If an intermediate step fails to compile, fix the compilation errors before
proceeding. Do not commit broken code. All pre-commit checks (fmt, clippy,
test) must pass before each commit.


## Artifacts and Notes

The new GraphQL schema additions (for reference when writing E2E queries):

    input ImportBookInput {
      title: String!
      authorNames: [String!]!
      isbn: String!
      read: Boolean!
      owned: Boolean!
      priority: Int!
      format: BookFormat!
      store: BookStore!
    }

    type Mutation {
      # … existing mutations …
      importBooks(books: [ImportBookInput!]!): [Book!]!
    }


## Interfaces and Dependencies

In `src/domain/repository/import_books_repository.rs`, the following types
must be defined and exported:

    pub struct ImportBookInput {
        pub book_id: BookId,
        pub title: BookTitle,
        pub author_names: Vec<AuthorName>,
        pub isbn: Isbn,
        pub read: ReadFlag,
        pub owned: OwnedFlag,
        pub priority: Priority,
        pub format: BookFormat,
        pub store: BookStore,
        pub created_at: OffsetDateTime,
        pub updated_at: OffsetDateTime,
    }

    #[automock]
    #[async_trait]
    pub trait ImportBooksRepository: Send + Sync + 'static {
        async fn import(
            &self,
            user_id: &UserId,
            books: Vec<ImportBookInput>,
        ) -> Result<Vec<Book>, DomainError>;
    }

In `src/infrastructure/import_books_repository.rs`:

    #[derive(Debug, Clone)]
    pub struct PgImportBooksRepository {
        pool: PgPool,
    }

    impl PgImportBooksRepository {
        pub fn new(pool: PgPool) -> Self { Self { pool } }
    }

    #[async_trait]
    impl ImportBooksRepository for PgImportBooksRepository { … }

In `src/use_case/interactor/book.rs`:

    pub struct ImportBooksInteractor<IBR> {
        import_books_repository: IBR,
    }

    impl<IBR: ImportBooksRepository> ImportBooksUseCase
        for ImportBooksInteractor<IBR> { … }

In `src/use_case/traits/book.rs`:

    #[automock]
    #[async_trait]
    pub trait ImportBooksUseCase: Send + Sync + 'static {
        async fn import(
            &self,
            user_id: &str,
            books: Vec<ImportBookEntryDto>,
        ) -> Result<Vec<BookDto>, UseCaseError>;
    }

The `MutationInteractor` signature after this change:

    pub struct MutationInteractor<
        RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC, RBUC, RAUC, IBUC
    > { … }

The `MI` type alias after this change:

    pub type MI = MutationInteractor<
        RegisterUserInteractor<PgUserRepository>,
        CreateBookInteractor<PgBookRepository>,
        UpdateBookInteractor<PgBookRepository>,
        DeleteBookInteractor<PgBookRepository>,
        CreateAuthorInteractor<PgAuthorRepository>,
        UpdateAuthorInteractor<PgAuthorRepository>,
        DeleteAuthorInteractor<PgAuthorRepository>,
        RestoreBookInteractor<PgBookRepository, PgBookEventRepository>,
        RestoreAuthorInteractor<PgAuthorRepository, PgAuthorEventRepository>,
        ImportBooksInteractor<PgImportBooksRepository>,
    >;
