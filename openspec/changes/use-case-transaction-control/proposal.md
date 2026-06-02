## Why

Currently, every repository method (`create`, `update`, `delete`) starts and commits its own PostgreSQL transaction inside the infrastructure layer (`PgBookRepository`, `PgAuthorRepository`). This makes it impossible for the use-case layer to orchestrate multiple repository operations within a single atomic transaction. As a result, a temporary `ImportBooksRepository` trait had to be introduced to bundle book and author creation together, and there is no clean path to add future cross-aggregate operations (e.g., author merge, bulk updates) without creating more ad-hoc "temporary" repositories.

## What Changes (Phase 1)

This change is the first phase of a multi-phase migration. Only `BookRepository` and `AuthorRepository` are affected in this phase.

- **BREAKING**: Add `&mut PgConnection` parameter to `BookRepository` and `AuthorRepository` trait methods.
- **BREAKING**: Update `PgBookRepository` and `PgAuthorRepository` to accept injected `&mut PgConnection` instead of owning `PgPool` and starting internal transactions.
- Move transaction boundaries (`pool.begin().await?` → `tx.commit().await?`) from these two infrastructure repositories into their respective use-case interactors.
- Remove the temporary `ImportBooksRepository` trait and `PgImportBooksRepository`; fold its logic into `ImportBooksInteractor` using the standard `BookRepository` + `AuthorRepository` + a single `PgPool`.
- Update event recording (`event_set` + `*_event` inserts) in `PgBookRepository` / `PgAuthorRepository` to occur inside the same transaction that the use-case layer manages.
- Update `AGENTS.md` "Event Recording" rule to reflect that transaction control now lives in the use-case layer while event recording remains an infrastructure-layer concern.
- Update all unit tests to continue using `mockall`-generated mocks; mocks simply ignore the `&mut PgConnection` argument.

## Future Phases

### Phase 2 — Remaining Repositories
Apply the same `&mut PgConnection` pattern to `UserRepository`, `BookEventRepository`, and `AuthorEventRepository`. Move transaction boundaries from `PgUserRepository`, `PgBookEventRepository`, and `PgAuthorEventRepository` into their respective use-case interactors. Update DI layer to pass `PgPool` where needed.

### Phase 3 — Cleanup & Standardization
- Revisit read-only query patterns: decide whether `QueryInteractor` should also receive `PgPool` for transactional reads, or keep `&self` repository calls for reads.
- Introduce a shared `Connection` abstraction if needed to reduce direct `sqlx` coupling in the domain layer.
- Audit all remaining `pool.begin()` calls in infrastructure to ensure none are left behind.

## Capabilities

### New Capabilities
- `use-case-transaction-control`: Enables the use-case layer to start, orchestrate, and commit database transactions across multiple repository calls.

### Modified Capabilities
<!-- No existing spec-level behavior changes; this is an architectural refactor. -->

## Impact

- **Domain layer**: `BookRepository` and `AuthorRepository` trait signatures change (add `&mut PgConnection`).
- **Infrastructure layer**: `PgBookRepository` and `PgAuthorRepository` drop `pool.begin()` from mutation methods. Event recording SQL stays inside each repository but runs on the injected connection.
- **Use-case layer**: Book and author mutation interactors gain a `pool: PgPool` field. `ImportBooksInteractor` now uses `BookRepository` + `AuthorRepository` instead of `ImportBooksRepository`.
- **Dependency injection**: Assemblers pass `pool` into book/author interactors that now require it.
- **Tests**: Unit tests continue using `MockBookRepository` / `MockAuthorRepository` with `always()` matchers for the new connection argument.
- **Documentation**: `AGENTS.md` event-recording rule updated.
