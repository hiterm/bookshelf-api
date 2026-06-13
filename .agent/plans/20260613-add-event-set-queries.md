# Add EventSet Queries

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with `.agent/PLANS.md`.

## Purpose / Big Picture

After this change, users can query their full change history at the event-set level via GraphQL. An "event set" is a single logical operation (e.g. "import_books", "create_book") that may span multiple entity changes in one database transaction. Before this change, users can only see events for a single book or author by ID. After this change, they can list all event sets with `{ eventSets { id operation createdAt } }` and drill into any one with `{ eventSet(id: "...") { id operation bookEvents { ... } authorEvents { ... } } }`.

This enables building a full audit trail UI: the user sees a chronological feed of every write operation they performed, and can expand any entry to see exactly which books and authors were created, updated, or deleted.

## Progress

- [x] Milestone 1+2: Domain + Infrastructure — EventSetRepository trait, find_by_event_set
  on BookEventRepository/AuthorEventRepository, PgEventSetRepository, and the matching
  infra impls plus DB-gated tests. Combined into one commit because adding a trait method
  breaks the existing Pg impls until they are implemented (the tree cannot compile, and so
  cannot pass `cargo test`, with domain-only changes).
  - [x] plan updated
- [ ] Milestone 3: Use-case DTO and trait — EventSetDto, EventSetDetailDto, and two new QueryUseCase methods.
  - [ ] plan updated
- [ ] Milestone 4: Use-case interactor — QueryInteractor gains ESR generic param and implements the two new methods.
  - [ ] plan updated
- [ ] Milestone 5: Presentation — EventSetEntry and EventSetDetail GraphQL objects; eventSets and eventSet resolvers; schema regenerated.
  - [ ] plan updated
- [ ] Milestone 6: DI — PgEventSetRepository wired into dependency_injection.
  - [ ] plan updated
- [ ] Milestone 7: E2E tests — e2e_event_sets test and eventSet assertion added to e2e_import_books.
  - [ ] plan updated

## Surprises & Discoveries

- Adding `find_by_event_set` to the `BookEventRepository`/`AuthorEventRepository` traits
  breaks compilation of the existing `Pg*` impls until those impls are added. A domain-only
  milestone therefore cannot pass `cargo test`, so Milestones 1 and 2 were combined into a
  single commit that keeps the tree green.

## Decision Log

- Decision: EventSetRepository uses a separate infra file rather than folding into the transaction module.
  Rationale: Consistent with existing pattern (PgBookEventRepository, PgAuthorEventRepository each have their own file). The transaction module focuses on write concerns; a query-only repository belongs alongside the other read repositories.
  Date/Author: 2026-06-13

- Decision: EventSetDetailDto is constructed by the use-case interactor by joining the EventSet with BookEvent and AuthorEvent lookups by event_set_id.
  Rationale: The infra layer is kept simple (three separate queries). Joining at the use-case level keeps SQL queries straightforward and avoids a complex multi-entity SQL join that would need custom row mapping.
  Date/Author: 2026-06-13

## Outcomes & Retrospective

Not yet written.

## Context and Orientation

The project is a Rust GraphQL API using async-graphql, sqlx (Postgres), and a layered architecture: domain → use-case → infrastructure → presentation.

Key files:

- `src/domain/entity/event_set.rs` — defines `EventSet` (id, user_id, operation, created_at) and `EventSetId`.
- `src/domain/repository/` — trait files for each repository. `book_event_repository.rs` and `author_event_repository.rs` already exist.
- `src/infrastructure/` — `PgBookEventRepository`, `PgAuthorEventRepository` are examples to follow.
- `src/use_case/dto/event.rs` — `BookEventDto`, `AuthorEventDto` exist. We add `event_set.rs`.
- `src/use_case/traits/query.rs` — `QueryUseCase` trait; extended with two new methods.
- `src/use_case/interactor/query.rs` — `QueryInteractor<UR, BR, AR, BER, AER>` implements `QueryUseCase`. Extended with a new `ESR` type parameter.
- `src/presentation/graphql/object.rs` — `BookEventEntry`, `AuthorEventEntry` exist; we add `EventSetEntry`, `EventSetDetail`.
- `src/presentation/graphql/query.rs` — GraphQL resolvers; two new resolvers added.
- `src/dependency_injection.rs` — type alias `QI` and `dependency_injection()` function; updated to include `PgEventSetRepository`.
- `e2e/tests/e2e.rs` — integration tests.

An "event set" in the domain represents one logical transaction boundary. `PgTransactionManager::begin` inserts one row into the `event_set` table. Each repository method participating in that transaction inserts a row into `book_event` or `author_event` with the same `event_set_id`.

The existing `EventSetRepository` trait does not yet exist — we create it. `BookEventRepository` and `AuthorEventRepository` both need a new `find_by_event_set` method.

## Plan of Work

Milestone 1 adds the domain repository trait for `EventSetRepository` and extends the two existing event repository traits with a `find_by_event_set` method. No infrastructure changes yet.

Milestone 2 creates `PgEventSetRepository` in `src/infrastructure/event_set_repository.rs`, implements `find_by_event_set` on `PgBookEventRepository` and `PgAuthorEventRepository`, and registers the new module in `src/infrastructure.rs`. DB-gated tests are included in each infra file.

Milestone 3 creates `src/use_case/dto/event_set.rs` with `EventSetDto` and `EventSetDetailDto`, registers it in `src/use_case/dto.rs`, and adds `list_event_sets` and `find_event_set` to the `QueryUseCase` trait.

Milestone 4 updates `QueryInteractor` to accept a 6th generic `ESR: EventSetRepository`, adds the `event_set_repository` field, implements both new methods, updates all existing test constructions to include `event_set_repository: MockEventSetRepository::new()`, and adds five new unit tests.

Milestone 5 adds `EventSetEntry` and `EventSetDetail` to `src/presentation/graphql/object.rs`, adds `event_sets` and `event_set` resolvers to `src/presentation/graphql/query.rs`, and regenerates `schema.graphql` by running `cargo run --bin gen_schema > schema.graphql`.

Milestone 6 updates `src/dependency_injection.rs` to import `PgEventSetRepository`, extend the `QI` type alias to 6 type params, and add the new repository to the `QueryInteractor` construction.

Milestone 7 adds the `e2e_event_sets` test to `e2e/tests/e2e.rs` and inserts an `eventSet` assertion block inside the existing `e2e_import_books` test.

## Concrete Steps

All commands run from `/home/user/bookshelf-api`.

Pre-commit checks (run before every commit):

    cargo fmt --check
    git add -A && cargo clippy --fix --allow-staged --all-targets -- -D warnings
    cargo test

Milestone 1 commit message: "Add EventSetRepository domain trait and find_by_event_set methods"
Milestone 2 commit message: "Add PgEventSetRepository and find_by_event_set impls"
Milestone 3 commit message: "Add EventSet DTOs and QueryUseCase methods"
Milestone 4 commit message: "Implement list_event_sets and find_event_set in QueryInteractor"
Milestone 5 commit message: "Add eventSets and eventSet GraphQL queries"
Milestone 6 commit message: "Wire PgEventSetRepository into DI"
Milestone 7 commit message: "Add e2e_event_sets test and eventSet assertion in e2e_import_books"

## Validation and Acceptance

Run `cargo test` after each milestone. All existing tests must pass.

After Milestone 5, run `cargo run --bin gen_schema > schema.graphql` and inspect the diff — it should contain `eventSets`, `eventSet`, `EventSetEntry`, `EventSetDetail`.

After Milestone 7, run `cargo test -p bookshelf-e2e --no-run` to confirm the e2e crate compiles. Full e2e test execution requires a running Postgres and app server.

## Idempotence and Recovery

Each milestone is committed independently. To restart from any milestone, check out the relevant commit and re-apply the steps from the next milestone onward.

## Artifacts and Notes

The `event_set` table schema (from existing migrations):

    CREATE TABLE event_set (
        id UUID PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES "user"(id),
        operation TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now()
    );

`book_event` and `author_event` both have an `event_set_id UUID NOT NULL REFERENCES event_set(id)` column and a `user_id TEXT NOT NULL` column, enabling per-user filtering.

## Interfaces and Dependencies

In `src/domain/repository/event_set_repository.rs`:

    #[automock]
    #[async_trait]
    pub trait EventSetRepository: Send + Sync + 'static {
        async fn find_all(&self, user_id: &UserId) -> Result<Vec<EventSet>, DomainError>;
        async fn find_by_id(
            &self,
            user_id: &UserId,
            event_set_id: &EventSetId,
        ) -> Result<Option<EventSet>, DomainError>;
    }

In `src/use_case/traits/query.rs` (additions):

    async fn list_event_sets(&self, user_id: &str) -> Result<Vec<EventSetDto>, UseCaseError>;
    async fn find_event_set(
        &self,
        user_id: &str,
        event_set_id: &str,
    ) -> Result<Option<EventSetDetailDto>, UseCaseError>;

In `src/use_case/interactor/query.rs`:

    pub struct QueryInteractor<UR, BR, AR, BER, AER, ESR> { ... }
