# Return Book and Author mutation event IDs

This ExecPlan is a living document. The sections `Progress`, `Surprises &
Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to
date as work proceeds. Maintain this document in accordance with
`.agent/PLANS.md`.

## Purpose / Big Picture

After this change, a GraphQL client that creates or updates a Book or Author can
read both `eventSetId`, which identifies the whole logical operation, and
`eventId`, which identifies the exact entity snapshot recorded by that
operation. The client can find the returned `eventId` in `bookEvents` or
`authorEvents` and pass it directly to `restoreBook` or `restoreAuthor`.
Delete, restore, import, and user registration payloads keep their existing
shape because those operations do not promise exactly one newly recorded
entity snapshot.

## Progress

- [x] Milestone 1: Propagate database-generated event IDs through repositories and create/update use cases, with unit tests.
  - [x] plan updated
- [x] Milestone 2: Expose required GraphQL `eventId` fields and regenerate the checked-in schema, with presentation tests.
  - [x] plan updated
- [x] Milestone 3: Prove history and restore interoperability with Book and Author E2E tests.
  - [x] plan updated
- [x] Milestone 4: Update architecture documentation and pass all mandatory validation.
  - [x] plan updated
- [x] Milestone 5: Replace bare create/update event ID results with the domain `EventId` newtype.
  - [x] plan updated

Work began on 2026-08-15 UTC.

## Surprises & Discoveries

- Observation: `PgBookRepository` already uses `INSERT ... RETURNING event_id`
  for create and update so it can populate `book_event_author`, but then returns
  `()` to its caller. `PgAuthorRepository` inserts equivalent events with
  `execute` and does not currently read the generated ID.
  Evidence: `src/infrastructure/book_repository.rs` binds the returned ID to
  `event_id`; `src/infrastructure/author_repository.rs` executes the event
  inserts without `RETURNING`.
- Observation: `BookRepository::create` is shared by single-Book creation and
  `ImportBooksInteractor`.
  Evidence: `src/use_case/interactor/book.rs` invokes the same method inside
  both transaction flows.
- Observation: The E2E suite requires a running API and rejects execution when
  `TEST_SERVER_URL` is absent; the documented Docker PostgreSQL, JWKS, and API
  setup runs successfully in this workspace.
  Evidence: the initial run failed only with the missing environment variable;
  after starting the documented services, 6 history and 6 restore tests passed.

## Decision Log

- Decision: Change Book and Author repository `create` and `update` methods to
  return `Result<EventId, DomainError>` containing the inserted entity event ID.
  Rationale: The database-generated ID is known atomically at insertion time;
  a later history lookup adds work and can select the wrong row under
  concurrency.
  Date/Author: 2026-08-15, Codex.
- Decision: Add `EntityMutationResultDto<T>` for Book and Author create/update,
  while retaining `MutationResultDto<T>` for delete, restore, and import.
  Rationale: A required `EventId` expresses the one-event guarantee without making
  unrelated operations carry an optional or misleading identifier.
  Date/Author: 2026-08-15, Codex.
- Decision: Let `ImportBooksInteractor` explicitly ignore each event ID returned
  by `BookRepository::create`.
  Rationale: Import creates multiple Book and possibly Author events, so no one
  event ID truthfully represents the mutation payload.
  Date/Author: 2026-08-15, Codex.
- Decision: Represent create/update repository results and entity mutation DTO
  fields with `EventId` instead of bare `i64` values.
  Rationale: The newtype makes the identifier's meaning explicit and prevents
  unrelated numeric IDs from satisfying these internal contracts. PostgreSQL
  and GraphQL remain the conversion boundaries.
  Date/Author: 2026-08-15, Codex.

## Outcomes & Retrospective

The feature is complete. Repository event inserts return their database IDs as
the domain `EventId` newtype, create/update use cases preserve those IDs through
commit, and GraphQL exposes them as required IDs without changing other
mutation contracts. The generated schema is current, all 159 Rust unit tests
pass, and 12 history/restore E2E tests pass against local PostgreSQL. Formatting
and warning-denying clippy validation also pass.

## Context and Orientation

The event log has an `event_set` row for one user-visible operation and one or
more `book_event` or `author_event` rows for entity snapshots. The
`TransactionManager` in `src/domain/repository/transaction.rs` opens and commits
the transaction. Book and Author repository traits in
`src/domain/repository/book_repository.rs` and
`src/domain/repository/author_repository.rs` receive that transaction and both
change live state and insert events. PostgreSQL implementations live in
`src/infrastructure/book_repository.rs` and
`src/infrastructure/author_repository.rs`.

The interactors in `src/use_case/interactor/book.rs` and
`src/use_case/interactor/author.rs` orchestrate each transaction. Their public
results are declared in `src/use_case/dto/mutation.rs` and flow through the
combined mutation use case to `src/presentation/graphql/mutation.rs`. GraphQL
payload objects are declared in `src/presentation/graphql/object.rs`; the
generated schema snapshot is `schema.graphql`.

Repository and interactor tests are Rust unit or database-backed tests in their
source modules. GraphQL E2E tests live under `e2e/tests`; history and restore
coverage is concentrated in `graphql_history.rs` and `graphql_restore.rs`.

## Plan of Work

First, make repository `create` and `update` return the exact inserted event ID.
For Book, return the ID already read for the event-author join insert. For
Author, add `RETURNING event_id`, fetch the scalar ID, and return it. Update
mocks and database tests so both create and update prove the returned ID points
to the expected event. Add a generic `EntityMutationResultDto<T>` with `value`,
`event_set_id`, and `event_id`; use it only for Book and Author create/update.
Capture the event ID before commit and construct the successful result only
after commit. Existing failure tests must show that repository or commit errors
return `Err`, not a partially populated result.

Second, add `event_id: ID` to `BookMutationPayload` and
`AuthorMutationPayload`, extend their constructors, and pass
`ID(result.event_id.to_string())` from all four resolvers. Update schema-focused
and resolver tests, generate `schema.graphql` using the repository's existing
schema generation path, and assert both fields render as `eventId: ID!` while
delete, restore, import, and registration payloads do not acquire the field.

Third, update E2E operations for create and update to request `eventId`. For
each entity kind, verify the returned ID exists in history, shares the returned
event-set ID, and is accepted by the matching restore mutation. Reuse existing
authentication and database fixtures so the tests validate the production
repository-to-GraphQL path.

Finally, update `docs/architecture/event-recording.md` to define the two IDs and
the multi-event exclusion. Add a `CHANGELOG.md` entry only if inspection shows
that unreleased API changes are recorded there. Run formatting, lint, all Rust
tests, and the relevant E2E suites. Update this plan and the OpenSpec task list
at every completed milestone.

## Concrete Steps

Run all commands from the repository root
`/home/hiterm/.codex/worktrees/2868aaed-3a25-4349-8dc8-23d46957f293/bookshelf-api`.

Inspect changes throughout with:

    git --no-pager diff

Run focused unit tests after each layer is changed using test-name filters found
in the edited modules. Generate or refresh `schema.graphql` with the existing
repository command discovered from CI or source tests. At the final milestone,
run exactly:

    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test --locked

Run relevant E2E tests with the package and feature/environment command already
used by this repository. If PostgreSQL or authentication fixtures are not
available, record the exact failing prerequisite and keep the E2E execution
task open rather than treating infrastructure absence as a passing test.

## Validation and Acceptance

Acceptance requires the generated schema to contain required `eventId: ID!`
fields on `BookMutationPayload` and `AuthorMutationPayload`. Unit tests must
show all four create/update interactors return the repository-provided numeric
event ID only after a successful commit, and errors remain errors. E2E tests
must create and update each entity, locate the returned event ID in its history
with the same event-set ID, and successfully invoke restore with it. Delete,
restore, import, and registration schema contracts must remain unchanged. All
three mandatory Cargo commands must exit successfully.

## Idempotence and Recovery

Source edits and test commands are safe to repeat. Database-backed tests must
use their existing isolated fixture setup and cleanup. No migration or data
rewrite is required. If schema regeneration changes unrelated output, inspect
the generator and retain only output produced from the current Rust schema; do
not manually invent generated definitions. Never discard unrelated working-tree
changes.

## Artifacts and Notes

The normative OpenSpec artifacts are under
`openspec/changes/return-entity-event-ids/`. The architecture invariant is in
`docs/architecture/event-recording.md`, and the physical event table schema is
in `docs/database.md`.

## Interfaces and Dependencies

At completion, `src/use_case/dto/mutation.rs` defines:

    pub struct EntityMutationResultDto<T> {
        pub value: T,
        pub event_set_id: String,
        pub event_id: EventId,
    }

`BookMutationResultDto` and `AuthorMutationResultDto` alias this type. The Book
and Author repository traits return `Result<EventId, DomainError>` from `create`
and `update`. No new crate dependency is required. The GraphQL layer uses
`async_graphql::ID` and converts the signed 64-bit database ID to a decimal
string.

Plan revision note (2026-08-15): Initial plan created from the approved feature
description, repository architecture, and OpenSpec design before code changes.

Plan revision note (2026-08-15): Marked the first three milestones complete
after focused unit tests, schema regeneration, E2E compilation, and 12 live E2E
tests passed; recorded the E2E environment requirement and evidence.

Plan revision note (2026-08-15): Marked final validation complete after
`cargo fmt --check`, warning-denying clippy, and all Rust tests passed; updated
the retrospective with final evidence.

Plan revision note (2026-08-15): Replaced bare create/update event ID results
with `EventId`, keeping raw `i64` handling at the PostgreSQL boundary and
decimal string conversion at the GraphQL boundary.
