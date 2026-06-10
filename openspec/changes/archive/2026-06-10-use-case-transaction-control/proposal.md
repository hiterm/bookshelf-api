## Why

Currently, every repository method (`create`, `update`, `delete`) starts and commits its own PostgreSQL transaction inside the infrastructure layer (`PgBookRepository`, `PgAuthorRepository`). This makes it impossible for the use-case layer to orchestrate multiple repository operations within a single atomic transaction. As a result, a temporary `ImportBooksRepository` trait had to be introduced to bundle book and author creation together, and there is no clean path to add future cross-aggregate operations (e.g., author merge, bulk updates) without creating more ad-hoc "temporary" repositories.

## What Changes (Phase 1)

This change is the first phase of a multi-phase migration. Only `BookRepository` and `AuthorRepository` are affected in this phase.

- **Strategy — Expand & Contract**: Add `*_conn(&mut PgConnection, …)` variants alongside existing methods, migrate interactors one by one, then remove old methods and rename. This keeps the codebase compilable and test-passing at every commit.
- Add `*_conn` variants (`create_conn`, `find_by_id_conn`, `find_all_conn`, `update_conn`, `delete_conn`, `restore_conn`) to `BookRepository` and `AuthorRepository`.
- Update `PgBookRepository` and `PgAuthorRepository` so new `*_conn` methods accept injected `&mut PgConnection`; old methods become thin wrappers that delegate to `*_conn` inside `pool.begin() … tx.commit()`.
- Move transaction boundaries (`pool.begin().await?` → `tx.commit().await?`) from infrastructure repositories into use-case interactors one at a time.
- Remove the temporary `ImportBooksRepository` trait and `PgImportBooksRepository`; fold its logic into `ImportBooksInteractor` using standard `BookRepository` + `AuthorRepository` + `PgPool`.
- Keep event recording (`event_set` + `*_event` inserts) inside `PgBookRepository` / `PgAuthorRepository`, but execute on the injected connection so it shares the use-case transaction.
- Update `AGENTS.md` "Event Recording" rule to clarify that transaction control lives in the use-case layer while event recording stays in the infrastructure layer.
- Unit tests continue using `mockall` mocks; `*_conn` mock expectations use `always()` for the connection argument.

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
