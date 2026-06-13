# Remove ImportBooksRepository — use-case-controlled transactions

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

This plan must be maintained in accordance with `.agent/PLANS.md` (at the
repository root).

## Purpose / Big Picture

Today the bulk book import feature is served by a special, explicitly
"temporary" repository pair: the domain trait `ImportBooksRepository` and its
implementation `PgImportBooksRepository`. They exist only because the ordinary
repositories `PgBookRepository` and `PgAuthorRepository` each open their own
PostgreSQL transaction internally (one `pool.begin()` per mutating method).
That makes it impossible for the use-case layer to run a `Book` write and an
`Author` write inside the *same* transaction, which the import needs (all rows
of one import must commit or roll back together and share one event grouping).

After this change, transaction control lives in the use-case layer. A new
domain trait `TransactionManager` opens a transaction and hands it to the
repositories; every mutating repository method accepts that transaction as its
first argument. The `ImportBooksInteractor` then composes the ordinary
`BookRepository` and `AuthorRepository` inside one transaction, and the
temporary import repository is deleted entirely.

What someone can do after this change that they could not before: orchestrate
multiple repositories in a single transaction from the use-case layer, with no
special-case repository. Observable behavior is unchanged: the GraphQL
`importBooks` mutation still imports books atomically and all author/book
events of one import still share a single `event_set` row whose operation is
`import_books`. Existing unit tests, infrastructure DB tests, and the
`e2e_import_books` end-to-end test continue to pass.

"Transaction" here means a PostgreSQL `BEGIN … COMMIT` block: either every
write inside it lands in the database or none does. "event_set" is a table
that groups the audit events of one logical operation; every mutating
operation inserts exactly one `event_set` row and one or more `book_event` /
`author_event` rows referencing it.

## Progress

- [x] M0: Create this ExecPlan.
  - [x] plan updated
- [x] M1: Expand `EventSetOperation` to 9 variants + unit tests.
  - [x] plan updated
- [x] M2: Add domain `TransactionManager` trait.
  - [x] plan updated
- [x] M3: Add infra `PgTransaction` + `PgTransactionManager`.
  - [x] plan updated
- [x] M4: Migrate `BookRepository` + `PgBookRepository` + book interactors +
  `RestoreBookInteractor` + tests + DI.
  - [x] plan updated
- [x] M5: Migrate `AuthorRepository` (incl. `find_or_create_by_name`) + author
  interactors + `RestoreAuthorInteractor` + tests + DI.
  - [x] plan updated
- [x] M6: Rewrite `ImportBooksInteractor`; move `ImportBookInput` in; rewrite
  unit tests; update DI `MI`; add re-homed DB integration test.
  - [x] plan updated
- [x] M7: Delete `import_books_repository.rs` (domain + infra) + module decls.
  - [x] plan updated
- [x] M8: Docs — amend CLAUDE.md/AGENTS.md, append Decision Log, finalize plan.
  - [x] plan updated

## Surprises & Discoveries

- Observation: The `import_books` `event_set_operation` value is seeded by a
  *later* migration (`20260515154314_add_author_name_unique.sql`), not the
  original event-tables migration (`20260429040611_add_event_tables.sql`).
  All 9 operations are nonetheless present at runtime.
  Evidence: `grep -rln import_books migrations/` matches only the 20260515 file;
  the 20260429 file seeds the other 8.

- Observation: The `event_set` table columns are `(id, user_id, operation,
  created_at)` with `created_at` defaulting to `current_timestamp`. The eager
  INSERT in `PgTransactionManager::begin` therefore binds only `(id, user_id,
  operation)`, exactly matching every existing per-repo INSERT.
  Evidence: `migrations/20260429040611_add_event_tables.sql` lines 16-21.

- Observation: `DomainError` has `From<sqlx::Error>` (in
  `src/infrastructure/error.rs`), so `?` on sqlx calls inside
  `PgTransactionManager` works without extra mapping.
  Evidence: `src/infrastructure/error.rs`.

- Observation: `CLAUDE.md` is a symlink to `AGENTS.md` (an earlier combined
  shell command misreported this). Editing the real target `AGENTS.md` updates
  both, so the M8 amendment was written once to `AGENTS.md`. The Write/Edit
  tooling also refuses to write through the symlink, which confirmed it.
  Evidence: `ls -la CLAUDE.md` shows `CLAUDE.md -> AGENTS.md`;
  `readlink CLAUDE.md` prints `AGENTS.md`.

- Observation: Naming the accessor `PgTransaction::as_mut` trips
  `clippy::should_implement_trait` under `-D warnings`. Kept the name (the
  plan and repositories call `tx.as_mut()`) and suppressed the lint with an
  explanatory comment per CLAUDE.md, rather than implementing `std::AsMut`.
  Evidence: clippy emitted `methods called as_mut usually implement
  std::convert::AsMut`.

- Observation: Migrating `BookRepository` in M4 broke not only the book infra
  DB tests but also `book_event_repository.rs` and `author_repository.rs` DB
  test modules, which call `book_repository.create` to set up fixtures. They
  had to be migrated to a local begin/commit helper in the same commit so the
  `test-with-database` build stays green.
  Evidence: `cargo check --features test-with-database --all-targets` reported
  8 `E0061` errors across those two files before the helpers were added.

- Observation: `find_or_create_by_name` needs a row struct distinct from
  `AuthorSnapshotRow` because it selects `id` (the DB-generated id after an
  ON CONFLICT insert) rather than `name`. Added `AuthorIdSnapshotRow`
  (id, yomi, created_at, updated_at) to mirror the old PgImportBooksRepository
  SELECT exactly.
  Evidence: the original import repo selected `id, yomi, created_at, updated_at`.

- Observation: Migrating `AuthorRepository` in M5 likewise broke the
  `book_repository.rs`, `book_event_repository.rs`, and (soon-deleted)
  `import_books_repository.rs` DB test modules that create authors as fixtures.
  Their `prepare_authors`/inline author creates were routed through a local
  begin/commit helper in the same commit to keep the gated build green.
  Evidence: `cargo build` reported `E0061`/`E0308` errors at those call sites
  until the helpers were threaded through.

- Deviation: The re-homed `import_rolls_back_on_failure` integration test no
  longer forces a duplicate-book_id collision the way the old
  PgImportBooksRepository test did, because the interactor now generates fresh
  book UUIDs internally so a caller cannot supply a colliding id. The test was
  re-expressed to assert the transactional invariant that is still reachable:
  a domain validation failure (an empty title) occurs BEFORE `begin`, so no
  book/author/event_set rows are persisted. True mid-transaction DB rollback
  remains covered by the repository-level DB tests (e.g. the book repository's
  failed-update test). Recorded here per the plan's design-blocker guidance.

## Decision Log

- Decision: Model the transaction with a domain trait `TransactionManager` that
  has an associated type `Transaction`, not a generic parameter.
  Rationale: mockall 0.14 pins an associated type via
  `#[automock(type Transaction = ();)]`, and an associated-type equality bound
  (`BR: BookRepository<Transaction = TM::Transaction>`) composes cleanly with
  the static-generics dependency injection already used in this codebase. A
  generic parameter would force every interactor and the DI aliases to thread
  an extra type parameter awkwardly.
  Date/Author: 2026-06-12 / Claude

- Decision: Insert the `event_set` row eagerly inside
  `PgTransactionManager::begin`, making it the single place event_set rows are
  created. Repositories read `tx.event_set_id()` instead of generating their
  own UUID.
  Rationale: A single source of truth for the event_set id is what lets the
  import share one event_set across all author/book events. Eager insertion is
  safe because `book_event`/`author_event` FK-reference `event_set` (never the
  reverse), and an early-return failure (e.g. `AuthorRepository::delete`'s
  `HasAssociatedBooks`) simply rolls the event_set row back with the rest of the
  transaction. There is no explicit rollback method: sqlx rolls a transaction
  back on drop, so an early `?` return after `begin()` is safe.
  Date/Author: 2026-06-12 / Claude

- Decision: All mutating methods (create/update/delete/restore) of both
  repositories migrate to transaction-accepting signatures in one unified
  design, rather than only the import path.
  Rationale: User decision. One consistent signature is easier to reason about
  than a mix of pool-opening and transaction-accepting methods.
  Date/Author: 2026-06-12 / Claude

- Decision: The import keeps a single shared `event_set` (operation
  `import_books`) spanning all author and book events.
  Rationale: User decision. The `e2e_import_books` end-to-end test asserts a
  single shared `eventSetId`, and preserving it keeps behavior byte-for-byte
  equivalent at the database level.
  Date/Author: 2026-06-12 / Claude

- Decision: Add `AuthorRepository::find_or_create_by_name(tx, user_id, name)
  -> AuthorId` to cover the import's author-resolution path, preserving the
  existing `INSERT … ON CONFLICT (user_id, name) DO NOTHING` + `SELECT` +
  record-an-author_event-only-when-newly-inserted behavior.
  Rationale: The import must resolve author names to ids and record a create
  event exactly once per newly inserted author. Putting this in
  `AuthorRepository` lets the interactor compose it with `BookRepository::create`
  inside one transaction. Author-name deduplication moves up into the
  interactor (a name→AuthorId `HashMap`).
  Date/Author: 2026-06-12 / Claude

- Decision: Expand `EventSetOperation` from the single `ImportBooks` variant to
  all 9 (CreateBook, UpdateBook, DeleteBook, RestoreBook, CreateAuthor,
  UpdateAuthor, DeleteAuthor, RestoreAuthor, ImportBooks) with an `as_str`
  round-trip, resolving the existing TODO in `src/domain/entity/event.rs`.
  Rationale: `begin` takes an `EventSetOperation` so the interactor chooses the
  operation; this is the only event concept that crosses into the use-case
  layer. Per-event mechanics stay in infrastructure.
  Date/Author: 2026-06-12 / Claude

## Outcomes & Retrospective

All milestones M0-M8 are complete. `ImportBooksRepository` and
`PgImportBooksRepository` are deleted. Transaction control now lives in the
use-case layer via the domain `TransactionManager` trait and its
infrastructure implementation `PgTransactionManager`/`PgTransaction`. Every
mutating repository method accepts `&mut Self::Transaction` and reads
`tx.event_set_id()`; the `event_set` INSERT was lifted out of each repository
method into the single eager insert in `PgTransactionManager::begin`. The bulk
import is composed from `BookRepository` + `AuthorRepository` +
`TransactionManager` and preserves the single shared `import_books` event_set,
matching the original DB-level behavior. `EventSetOperation` now has all nine
variants, resolving the prior TODO. CLAUDE.md/AGENTS.md and the change-history
Decision Log were amended to describe the new boundary.

Verification in this environment: `cargo fmt --check` clean,
`cargo clippy --all-targets -- -D warnings` clean, `cargo test` = 129 passed,
and `cargo check --features test-with-database --all-targets` compiles.

What remains for the user to run locally (no PostgreSQL here):
`cargo test --features test-with-database` (infra DB tests, the new
`find_or_create_by_name` tests, and the re-homed import integration tests) and
the `e2e_import_books` end-to-end test (docker compose + JWKS server per the
README), which must still observe a single shared `eventSetId`.

Lessons: a trait-signature change ripples into every cross-module test that
used the repository as a fixture; bundling those callers into the same
milestone commit was necessary to keep both the default and
`test-with-database` builds green. The clippy `should_implement_trait` lint on
`PgTransaction::as_mut` and the `CLAUDE.md -> AGENTS.md` symlink were the two
small surprises; both are recorded above.

## Context and Orientation

This is a Rust (edition 2024) GraphQL API backed by PostgreSQL via sqlx 0.8.
The architecture is layered: domain (`src/domain`), use-case (`src/use_case`),
infrastructure (`src/infrastructure`), presentation (`src/presentation`).

Key files for this task, by full path:

- `src/domain/entity/event.rs` — `EventSetOperation` and `EventOperation`
  enums. Has a TODO to migrate per-operation strings here.
- `src/domain/entity/event_set.rs` — `EventSetId` value object.
- `src/domain/repository/book_repository.rs`,
  `src/domain/repository/author_repository.rs` — repository traits, each
  annotated with `#[automock]` then `#[async_trait]`.
- `src/domain/repository/import_books_repository.rs` — the temporary trait and
  the `ImportBookInput` struct (to be deleted; struct moves into the import
  interactor).
- `src/domain/repository.rs` — the repository module declaration file.
- `src/infrastructure/book_repository.rs`,
  `src/infrastructure/author_repository.rs`,
  `src/infrastructure/import_books_repository.rs` — Pg implementations.
- `src/infrastructure.rs` — infrastructure module declaration file.
- `src/infrastructure/error.rs` — `From<sqlx::Error> for DomainError`.
- `src/use_case/interactor/book.rs`, `src/use_case/interactor/author.rs`,
  `src/use_case/interactor/event.rs` — interactors (incl. restore).
- `src/use_case/interactor.rs` — interactor module declaration file.
- `src/dependency_injection.rs` — wires Pg repos into interactors; defines the
  `QI` and `MI` type aliases.
- `migrations/20260429040611_add_event_tables.sql` — `event_set` schema and 8
  seeded operations; `migrations/20260515154314_add_author_name_unique.sql`
  seeds `import_books` and the `(user_id, name)` unique constraint.

Term definitions:

- "Interactor": a use-case implementation struct (e.g.
  `CreateBookInteractor`) that orchestrates domain entities and repositories.
- "DI": dependency injection — `src/dependency_injection.rs` constructs the
  concrete object graph.
- "mockall `#[automock]`": generates a `Mock<Trait>` type for unit tests. The
  attribute must appear ABOVE `#[async_trait]` to match the existing ordering.

## Plan of Work

### D1. Domain `TransactionManager` trait (new file `src/domain/repository/transaction.rs`)

    use async_trait::async_trait;
    use mockall::automock;

    use crate::domain::{
        entity::{event::EventSetOperation, user::UserId},
        error::DomainError,
    };

    #[automock(type Transaction = ();)]
    #[async_trait]
    pub trait TransactionManager: Send + Sync + 'static {
        type Transaction: Send;
        async fn begin(
            &self,
            user_id: &UserId,
            operation: EventSetOperation,
        ) -> Result<Self::Transaction, DomainError>;
        async fn commit(&self, tx: Self::Transaction) -> Result<(), DomainError>;
    }

`type Transaction: Send` is mandatory so `async_trait` futures are `Send`.
No explicit rollback: sqlx `Transaction` rolls back on drop. Register the
module in `src/domain/repository.rs`.

### D2. Infra `PgTransaction` + `PgTransactionManager` (new file `src/infrastructure/transaction.rs`)

`PgTransaction` wraps `sqlx::Transaction<'static, Postgres>` and a `Uuid`
event_set id, exposing `event_set_id() -> Uuid`, `as_mut() -> &mut
PgConnection`, and `commit(self)`. `PgTransactionManager { pool: PgPool }`
derives `Clone`. `begin` does `pool.begin()`, generates a UUID, inserts the
event_set row (`INSERT INTO event_set (id, user_id, operation) VALUES ($1, $2,
$3)` binding `operation.as_str()`), and returns the wrapper. Register the
module in `src/infrastructure.rs`.

### D3. Repository trait signatures

Mutating methods gain `tx: &mut Self::Transaction` as the first parameter; read
methods are unchanged (still pool-based, repos keep their `pool` field). Both
traits add `type Transaction: Send;` and switch to
`#[automock(type Transaction = ();)]`. `AuthorRepository` additionally gains
`find_or_create_by_name(&self, tx, user_id, name: &AuthorName) ->
Result<AuthorId, DomainError>`.

### D4. Interactors

Each mutating interactor gains a `transaction_manager: TM` field with bound
`TM: TransactionManager` and `XR: XRepository<Transaction = TM::Transaction>`.
Body: build the entity / validate, `begin(op)`, call repo method(s) with
`&mut tx`, `commit(tx)`. `UpdateBookInteractor` calls pool-based `find_by_id`
BEFORE `begin`. `RestoreBookInteractor`/`RestoreAuthorInteractor` likewise read
the event BEFORE `begin`.

### D5. `ImportBooksInteractor<BR, AR, TM>`

Validation and DTO→input mapping happen BEFORE `begin`. Then `begin(ImportBooks)`
→ per unique author name `find_or_create_by_name` building a name→AuthorId
HashMap → per book build `Book` and `book_repository.create(&mut tx, …)` →
`commit`. `ImportBookInput` moves into this module as a private struct.

### D7. DI

`let transaction_manager = PgTransactionManager::new(pool.clone());` cloned into
each mutating interactor. `MI` gains `PgTransactionManager` on every mutating
interactor; `ImportBooksInteractor<PgBookRepository, PgAuthorRepository,
PgTransactionManager>` replaces `ImportBooksInteractor<PgImportBooksRepository>`.

## Concrete Steps

Run from the repository root `/home/user/bookshelf-api`.

Before each non-doc-only commit, run the mandatory checks (per CLAUDE.md):

    cargo fmt --check
    cargo clippy --fix --all-targets -- -D warnings
    cargo test

For commits touching DB-gated tests, additionally confirm they still compile:

    cargo check --features test-with-database --all-targets

## Validation and Acceptance

- Unit tests: `cargo test` passes; mocks pin `Transaction = ()`. Each mutating
  interactor test adds `MockTransactionManager` with
  `expect_begin().returning(|_, _| Ok(()))` and
  `expect_commit().returning(|_| Ok(()))`. Validation-failure tests keep bare
  mocks (no expectations).
- Infra + import DB tests: gated behind `test-with-database`; cannot run
  without PostgreSQL. They must compile (`cargo check --features
  test-with-database --all-targets`). Run locally with
  `cargo test --features test-with-database`.
- E2E: `e2e_import_books` (user-run, needs docker compose + JWKS server per
  README) must still observe a single shared eventSetId.

## Idempotence and Recovery

Each milestone is an additive or self-contained change committed separately. If
a milestone fails to compile, fix it before committing. A trait-signature
change (M4/M5) cannot compile half-migrated, so its repo impl, interactors,
tests, and DI are bundled in one commit.

## Interfaces and Dependencies

- `crate::domain::repository::transaction::TransactionManager` (new trait).
- `crate::infrastructure::transaction::{PgTransaction, PgTransactionManager}`
  (new types).
- `mockall = "0.14.0"`, `async-trait`, `sqlx = 0.8`, `uuid`.

## Revision Notes

- 2026-06-12: Initial authoring (M0). Adapted from the approved design
  (D1-D7, M0-M8) into ExecPlan format. Reason: CLAUDE.md requires complex
  refactors to use an ExecPlan stored in `.agent/plans/`.
- 2026-06-12: Finalized at M8. Marked all milestones complete, wrote Outcomes &
  Retrospective, corrected the `CLAUDE.md`/`AGENTS.md` symlink observation, and
  recorded the import rollback-test deviation. Reason: the implementation is
  finished and the living-document sections must reflect the final state.
