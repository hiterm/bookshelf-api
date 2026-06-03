## 1. Preparation

- [ ] 1.1 Update `AGENTS.md` "Event Recording" rule to clarify that transaction control moves to the use-case layer while event recording stays in the infrastructure layer
- [ ] 1.2 Verify `sqlx` is already a dependency; confirm `mockall` version supports `&mut PgConnection` arguments (already verified)

## 2. Expand — Add Parallel `*_conn` Methods to Domain Layer

**Goal**: Add `*_conn` variants without breaking any existing code. Every commit must compile and pass tests.

- [ ] 2.1 Update `BookRepository` trait: add `*_conn` variants (`create_conn`, `find_by_id_conn`, `find_all_conn`, `update_conn`, `delete_conn`, `restore_conn`) with `conn: &mut PgConnection` as first parameter. Leave existing methods untouched.
- [ ] 2.2 Update `AuthorRepository` trait: add `*_conn` variants (`create_conn`, `find_by_id_conn`, `find_all_conn`, `find_by_ids_as_hash_map_conn`, `update_conn`, `delete_conn`, `restore_conn`). Leave existing methods untouched.
- [ ] 2.3 Run `cargo check` — must pass with zero errors.
- [ ] 2.4 Run `cargo test` — must pass (no callers use the new methods yet, so behavior is unchanged).

## 3. Expand — Implement `*_conn` in Infrastructure Layer + Make Old Methods Delegate

**Goal**: New methods do the real work on injected `conn`; old methods become thin wrappers.

- [ ] 3.1 Update `PgBookRepository`:
  - Implement `*_conn` methods using injected `&mut PgConnection` (extract logic from current methods).
  - Make old `create`, `update`, `delete`, `restore` delegate to `*_conn` inside `pool.begin() … tx.commit()`.
  - Make old `find_by_id`, `find_all` delegate to `*_conn` (acquire a connection and call the new method).
- [ ] 3.2 Update `PgAuthorRepository`: same pattern.
- [ ] 3.3 Run `cargo test` — must pass. Behavior is identical; only routing has changed.

## 4. Migrate — Move Transaction Control to Use-Case Interactors (One at a Time)

**Goal**: Each interactor gains `pool: PgPool`, starts its own transaction, and calls `*_conn`. Migrate one interactor per logical commit.

- [ ] 4.1 Update `CreateBookInteractor`: add `pool: PgPool` field, call `book_repository.create_conn(&mut tx, …)` inside `pool.begin() … tx.commit()`.
- [ ] 4.2 Update `UpdateBookInteractor`: wrap `find_by_id_conn` + `update_conn` in a transaction.
- [ ] 4.3 Update `DeleteBookInteractor`: wrap `delete_conn` in a transaction.
- [ ] 4.4 Update `CreateAuthorInteractor`: wrap `create_conn` in a transaction.
- [ ] 4.5 Update `UpdateAuthorInteractor`: wrap `update_conn` in a transaction.
- [ ] 4.6 Update `DeleteAuthorInteractor`: wrap `delete_conn` in a transaction.
- [ ] 4.7 Update `RestoreBookInteractor`: wrap `restore_conn` in a transaction.
- [ ] 4.8 Update `RestoreAuthorInteractor`: wrap `restore_conn` in a transaction.
- [ ] 4.9 After each sub-task above, run `cargo test` before proceeding.

## 5. Migrate — Rewrite ImportBooksInteractor & Remove ImportBooksRepository

**Goal**: `ImportBooksInteractor` uses `BookRepository` + `AuthorRepository` + `PgPool` directly, inside a single transaction.

- [ ] 5.1 Rewrite `ImportBooksInteractor`: replace `ImportBooksRepository` dependency with `BookRepository` + `AuthorRepository` + `PgPool`. In `import()`, begin a transaction, call `author_repository.create_conn` and `book_repository.create_conn` for each book, commit.
- [ ] 5.2 Update `MutationInteractor` constructor to accept and pass `PgPool` for `ImportBooksInteractor`.
- [ ] 5.3 Remove `ImportBooksRepository` trait (`src/domain/repository/import_books_repository.rs`).
- [ ] 5.4 Remove `PgImportBooksRepository` (`src/infrastructure/import_books_repository.rs`).
- [ ] 5.5 Run `cargo test` — must pass.

## 6. Migrate — Update QueryInteractor

**Goal**: `QueryInteractor` also uses `*_conn` methods so reads happen on acquired connections.

- [ ] 6.1 Add `pool: PgPool` to `QueryInteractor`.
- [ ] 6.2 Update `find_book_by_id`, `find_all_books` to acquire a connection and call `*_conn`.
- [ ] 6.3 Update `find_author_by_id`, `find_all_authors`, `find_author_by_ids_as_hash_map` similarly.
- [ ] 6.4 Run `cargo test` — must pass.

## 7. Dependency Injection Layer

- [ ] 7.1 Update `src/dependency_injection.rs` to pass `pool` into all interactors that now require it.
- [ ] 7.2 Remove `PgImportBooksRepository` instantiation and wiring.
- [ ] 7.3 Update type aliases (`QI`, `MI`, etc.) if generic parameter counts change.
- [ ] 7.4 Run `cargo test` — must pass.

## 8. Unit Tests

- [ ] 8.1 Update `src/use_case/interactor/book.rs` tests:
  - Add `always()` matcher for `&mut PgConnection` in `MockBookRepository` expectations for `*_conn` methods.
  - Rewrite `ImportBooksInteractor` tests to use `MockBookRepository` + `MockAuthorRepository` + a dummy `PgPool`.
- [ ] 8.2 Update `src/use_case/interactor/author.rs` tests: add `always()` matcher for `&mut PgConnection` in `MockAuthorRepository` expectations for `*_conn` methods.
- [ ] 8.3 Update `src/use_case/interactor/mutation.rs` tests: adjust for new `PgPool` parameter in interactors.
- [ ] 8.4 Update `src/use_case/interactor/event.rs` tests: adjust `MockBookRepository` / `MockAuthorRepository` expectations for `restore_conn`.
- [ ] 8.5 Update `src/use_case/interactor/query.rs` tests: adjust expectations for `*_conn` methods.
- [ ] 8.6 Run `cargo test --lib` and ensure all unit tests pass.

## 9. Database Integration Tests

- [ ] 9.1 Update `PgBookRepository` `#[sqlx::test]` tests to call `*_conn` methods, passing `&mut conn` obtained from `pool.acquire()`.
- [ ] 9.2 Update `PgAuthorRepository` `#[sqlx::test]` tests similarly.
- [ ] 9.3 Remove `PgImportBooksRepository` `#[sqlx::test]` tests (file no longer exists).
- [ ] 9.4 Run `cargo test --features test-with-database` and ensure all DB tests pass.

## 10. E2E Tests

- [ ] 10.1 Run E2E test suite against Docker Compose stack.
- [ ] 10.2 Fix any failures caused by behavioral changes (none expected since this is an internal refactor with no API changes).

## 11. Contract — Remove Old Methods and Rename

**Goal**: Clean up. Remove old methods that no longer have callers; rename `*_conn` back to original names.

- [ ] 11.1 Remove old methods (`create`, `find_by_id`, `find_all`, `update`, `delete`, `restore`) from `BookRepository` trait.
- [ ] 11.2 Rename `*_conn` → original names (`create_conn` → `create`, etc.) in `BookRepository`.
- [ ] 11.3 Remove old methods from `AuthorRepository` trait and rename `*_conn` → original names.
- [ ] 11.4 Update `PgBookRepository` and `PgAuthorRepository` implementations to match renamed trait methods.
- [ ] 11.5 Update all interactor calls from `*_conn` back to original names.
- [ ] 11.6 Update all unit test expectations from `expect_*_conn` back to `expect_*`.
- [ ] 11.7 Update DB integration tests similarly.
- [ ] 11.8 Run `cargo test` — must pass.

## 12. Code Quality & Final Verification

- [ ] 12.1 Run `cargo fmt --check` and fix formatting.
- [ ] 12.2 Run `cargo clippy --all-targets --features test-with-database -- -D warnings` and fix all warnings manually (do **not** use `cargo clippy --fix`).
- [ ] 12.3 Run `cargo test` (full suite) and ensure 100% pass rate.
- [ ] 12.4 Review `AGENTS.md` for consistency with the new architecture.

## Future Phases (Out of Scope for This Change)

### Phase 2 — Remaining Repositories
Apply the same Expand & Contract pattern to:
- `UserRepository` and `PgUserRepository`
- `BookEventRepository` and `PgBookEventRepository`
- `AuthorEventRepository` and `PgAuthorEventRepository`
Move transaction boundaries from these infrastructure repositories into their use-case interactors. Update DI layer accordingly.

### Phase 3 — Cleanup & Standardization
- Decide whether `QueryInteractor` should keep `PgPool` for transactional reads or switch to a lighter read pattern.
- Introduce a thin `Connection` abstraction if direct `sqlx` dependency in domain layer becomes problematic.
- Audit all remaining `pool.begin()` calls in infrastructure to ensure none are left behind.
- Re-evaluate `AGENTS.md` rules after full migration is complete.
