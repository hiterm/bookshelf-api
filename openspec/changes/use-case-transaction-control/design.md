## Context

The current architecture wraps every mutation (`create`, `update`, `delete`) inside the infrastructure layer (`PgBookRepository`, `PgAuthorRepository`). Each method calls `self.pool.begin().await?`, executes its SQL, records events, and commits. This makes the use-case layer a passive caller with no transaction scope control.

Because of this, cross-aggregate operations like "import books" (which creates authors and books atomically) could not be expressed as a composition of `BookRepository` + `AuthorRepository`. A temporary `ImportBooksRepository` trait and `PgImportBooksRepository` were introduced as a workaround, but this pattern does not scale to future operations (e.g., author merge, bulk updates).

We will migrate to a use-case-controlled transaction model in three phases. **This change covers Phase 1 only**.

## Goals / Non-Goals

**Goals (Phase 1):**
- Move transaction boundaries for `BookRepository` and `AuthorRepository` into the use-case layer.
- Remove the temporary `ImportBooksRepository` trait and merge its behavior back into standard `BookRepository` + `AuthorRepository` usage.
- Keep all existing unit tests using `mockall`-generated mocks without regressions.
- Ensure event recording in `PgBookRepository` / `PgAuthorRepository` remains atomic with the business operation by running inside the same transaction the use-case layer manages.

**Non-Goals (Phase 1):**
- No changes to `UserRepository`, `BookEventRepository`, or `AuthorEventRepository` (Phase 2).
- No database schema changes.
- No GraphQL API schema changes.
- No new user-facing features (this is an internal architectural refactor).
- No decision yet on whether `QueryInteractor` should receive `PgPool` (Phase 3).

## Decisions

### Decision: Phase 1 targets Book + Author repositories only
- **Rationale**: `BookRepository` and `AuthorRepository` are the most frequently used mutation repositories and are directly involved in the `ImportBooksRepository` workaround. Limiting Phase 1 to these two minimizes risk and validates the pattern before expanding.
- **Phase 2** will apply the same pattern to `UserRepository`, `BookEventRepository`, and `AuthorEventRepository`.

### Decision: Repository methods accept `&mut PgConnection` for mutations
- **Rationale**: `sqlx 0.8`'s `Executor<'c>` trait is not dyn-compatible and is effectively impossible to mock with `mockall`. `&mut PgConnection` is the concrete type that both `PoolConnection<<Postgres>` and `Transaction<'_, Postgres>` dereference to via `DerefMut`, so a single signature covers both pooled connections and transactions. It is also mockable.
- **Alternatives considered**: `Executor<'c>` (rejected: not mockable), `&mut Transaction<'_, Postgres>` (rejected: lifetime parameter on trait method complicates mockall but is possible; however `&mut PgConnection` is simpler and more general).

### Decision: Read-only methods (`find_by_id`, `find_all`) also accept `&mut PgConnection`
- **Rationale**: `find_by_id` is called inside `UpdateBookInteractor::update` to verify existence before updating, and this read must happen inside the same transaction. For signature uniformity, all methods on these two repositories will accept `&mut PgConnection`.

### Decision: Event recording stays inside each `Pg*` repository, but runs on the injected connection
- **Rationale**: `AGENTS.md` states event recording belongs exclusively in the infrastructure layer. By injecting `&mut PgConnection`, the repository can still insert into `event_set` and `*_event` within the same transaction the use-case started. The use-case interactor does not know that event recording is happening.
- **Impact on `AGENTS.md`**: The rule that "event recording belongs exclusively in the infrastructure layer" remains true. The only change is that the infrastructure layer no longer owns the transaction boundary.

### Decision: `ImportBooksRepository` is removed; `ImportBooksInteractor` uses `BookRepository` + `AuthorRepository` + `PgPool`
- **Rationale**: With connection injection, the interactor can create a transaction and call `author_repository.create(&mut tx, ...)` followed by `book_repository.create(&mut tx, ...)` inside the same `tx`. The temporary facade repository is no longer needed.
- **Note**: `ImportBooksInteractor` will gain a `pool: PgPool` field (or receive it via constructor) so it can call `pool.begin().await?`.

### Decision: Mock expectations use `always()` for the new `&mut PgConnection` argument
- **Rationale**: Unit tests do not need to assert on the connection argument. `mockall::predicate::always()` matches any argument, keeping existing test patterns intact.

## Risks / Trade-offs

- **[Risk]** Refactor surface area is still significant: all Book and Author repository methods, interactors, tests, and DI wiring change.
  - **Mitigation**: Compile and run tests after each layer (domain → infrastructure → use-case → DI) rather than changing everything at once.

- **[Risk]** `PgConnection` is a concrete type from `sqlx`, creating a dependency leak from infrastructure into domain layer.
  - **Mitigation**: This is accepted for Phase 1. The domain layer already depends on `async-trait` and `mockall`; adding `sqlx` as a dev-facing dependency for the trait definition is a minor increase in coupling. Phase 3 may introduce a thin `Connection` newtype wrapper if portability becomes a concern.

- **[Risk]** UserRepository, BookEventRepository, and AuthorEventRepository still start internal transactions, creating a temporary inconsistency where some repositories are injected and some are not.
  - **Mitigation**: Phase 2 will resolve this. The inconsistency is internal only; no user-visible behavior changes.

## Open Questions

1. *Resolved for Phase 1*: Only `BookRepository` and `AuthorRepository` will be changed. `UserRepository` and event repositories are deferred to Phase 2.

2. *Resolved for Phase 1*: `QueryInteractor` will **not** receive `PgPool` in this change. Reads will continue using `&self` repository calls. This decision will be revisited in Phase 3.
