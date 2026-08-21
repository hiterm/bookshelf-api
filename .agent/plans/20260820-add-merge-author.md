# Add the GraphQL author merge mutation

This ExecPlan is a living document maintained according to `.agent/PLANS.md`. It records the implementation of a GraphQL mutation that moves every book relationship from one author to another, deletes the source author, and records the entire logical operation atomically.

## Purpose / Big Picture

After this change an authenticated client can call `mergeAuthor(sourceAuthorId:, destinationAuthorId:)`. Every book associated with the source will instead reference the destination, the source will disappear, and the unchanged destination plus the merge event-set identifier will be returned. Book updates, source deletion, and destination participation will share one PostgreSQL transaction and one event set, so a failure cannot leave a partial merge.

## Progress

- [x] Milestone 1: Add domain operations, typed event metadata, transaction-aware repository APIs, migration, and repository coverage.
  - [x] plan updated
- [x] Milestone 2: Add the merge use case, deterministic author locking, restore guard, composition delegation, and unit tests.
  - [x] plan updated
- [x] Milestone 3: Expose `mergeAuthor` through GraphQL and dependency injection, update architecture documentation, and add E2E coverage.
  - [x] plan updated
- [x] Milestone 4: Run formatting, clippy, unit tests, and schema verification; record the unavailable external E2E environment.
  - [x] plan updated

Plan created 2026-08-20 UTC.

## Surprises & Discoveries

- Observation: The local environment does not define `TEST_SERVER_URL` or `DATABASE_URL`, so the server/PostgreSQL-backed E2E suite cannot be executed in this task environment.
  Evidence: `printenv TEST_SERVER_URL` and `printenv DATABASE_URL` both produced no value. The new E2E scenario remains compiled in `e2e/tests/graphql_authors.rs` for the configured CI environment.

- Observation: The first dependency resolution attempted inside the sandbox could not resolve crates.io; retrying with approved network access populated the cache and all subsequent checks ran normally.
  Evidence: the initial check reported `Could not resolve host: index.crates.io`; the approved retry downloaded the locked dependencies and completed.

## Decision Log

- Decision: Keep merge orchestration in a dedicated use-case interactor and use existing repository mutation methods for book updates and source deletion.
  Rationale: This preserves the transaction/event-set invariant while avoiding a merge-specific persistence service or domain method.
  Date/Author: 2026-08-20 / Codex

- Decision: Lock both authors by ascending UUID while preserving their original source and destination roles after lookup.
  Rationale: Concurrent inverse merges otherwise acquire row locks in opposite order and can deadlock.
  Date/Author: 2026-08-20 / Codex

## Outcomes & Retrospective

The `mergeAuthor` mutation is implemented from GraphQL through PostgreSQL persistence. It locks authors deterministically, moves and records every affected book, records typed source and destination author events in one event set, preserves the destination entity state, and rejects invalid restore semantics. The generated schema matches `schema.graphql`; formatting, clippy, and all 165 non-database unit tests pass. The external E2E scenario was added but could not be run locally because no test server or database URL was supplied.

## Context and Orientation

`src/domain/entity/event.rs` defines per-entity and event-set operation enums. Domain repository traits live in `src/domain/repository/`; PostgreSQL adapters in `src/infrastructure/` implement them using `PgTransaction`. Use-case traits and interactors in `src/use_case/` own transaction boundaries. `src/presentation/graphql/` converts GraphQL arguments and payloads, while `src/dependency_injection.rs` composes concrete implementations. The `e2e` crate exercises the running GraphQL API against PostgreSQL.

An event set groups all events produced by one logical user operation. A transaction-aware repository method accepts the same mutable transaction opened by the use case; it must not open another transaction or create another event set. A destination participation event is an event-only write: it records that the destination took part without modifying the author row or timestamp.

## Plan of Work

First extend operation enums and the migration lookup values. Introduce typed source-delete metadata and a write-only `NewAuthorEvent` input, then update PostgreSQL repositories so source-book lookup and destination event append use the caller's transaction. Preserve ordinary deletes by passing no metadata.

Next implement `MergeAuthorInteractor`. Validate IDs before opening the transaction, lock authors in UUID order, map the results back to source and destination, load source books, rebuild each book with only its author IDs changed, update each book, delete the source with typed merge metadata, append the destination participation event, and commit. Reject `merge_as_destination` as an author restore source. Delegate the new operation through `MutationInteractor`.

Finally add `MergeAuthorPayload`, the GraphQL resolver, and concrete dependency injection. Update event-recording and database documentation. Add unit tests for changed logic and GraphQL E2E tests that verify relationships, payload shape, event metadata, event-set membership, and no-book behavior.

## Concrete Steps

Work from `/home/hiterm/ghq/github.com/hiterm/bookshelf-api`. Inspect changes with `git --no-pager diff`. Run focused tests while implementing, then run the mandatory pre-commit commands in order: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`. Run the E2E test command documented by the existing `e2e` crate when its PostgreSQL environment is available.

## Validation and Acceptance

The generated GraphQL schema must contain `mergeAuthor(sourceAuthorId: ID!, destinationAuthorId: ID!): MergeAuthorPayload!`, and the payload must contain only `author` and `eventSetId`. A successful test scenario must show the source is no longer queryable, every former source book references the destination exactly once, other authors remain attached, and the destination timestamps are unchanged. Event-set detail must contain one update per moved book, one source delete with merge metadata, and one destination `merge_as_destination` event with source metadata. Invalid identical IDs and `merge_as_destination` restore attempts must return validation errors. Any repository or commit failure must avoid committing.

## Idempotence and Recovery

The migration inserts lookup values with `ON CONFLICT DO NOTHING`, so rerunning it is harmless. The merge itself is atomic but not idempotent after success because the source no longer exists; retrying a completed request returns not found without modifying the destination. Before commit, dropping a failed PostgreSQL transaction rolls back all partial writes.

## Artifacts and Notes

Validation transcripts will be added as milestones complete.

    cargo fmt --check
    # exit 0

    cargo clippy --all-targets --locked -- -D warnings
    Finished `dev` profile ...

    cargo test --locked
    test result: ok. 165 passed; 0 failed

    cargo run --quiet --bin gen_schema --locked | diff schema.graphql -
    # exit 0, no diff

    cargo check --locked -p bookshelf-e2e --tests
    Finished `dev` profile ...

Plan updated 2026-08-20 UTC after implementation and validation. The E2E execution limitation is recorded so a future contributor can run the committed scenario in the configured CI environment.

## Interfaces and Dependencies

Add `MergeAuthorInputDto`, `MergeAuthorUseCase::merge`, and `MutationUseCase::merge_author`, returning `MutationResultDto<AuthorDto>`. `MergeAuthorInteractor<AR, BR, AER, TM>` requires all three repositories to use `TM::Transaction`. Add `BookRepository::find_by_author_id_with_tx`, typed optional metadata to `AuthorRepository::delete`, and `AuthorEventRepository::append(&mut Transaction, &NewAuthorEvent)`. The destination event uses operation `merge_as_destination`; the event set uses `merge_author`.
