## 1. Preparation

- [ ] 1.1 Update `AGENTS.md` "Event Recording" rule to clarify that transaction control moves to the use-case layer while event recording stays in the infrastructure layer
- [ ] 1.2 Verify `sqlx` is already a dependency; confirm `mockall` version supports `&mut PgConnection` arguments (already verified)

## 2. Domain Layer — Update BookRepository and AuthorRepository Traits

- [ ] 2.1 Update `BookRepository` trait: add `conn: &mut PgConnection` as first parameter to `create`, `find_by_id`, `find_all`, `update`, `delete`, `restore`
- [ ] 2.2 Update `AuthorRepository` trait: add `conn: &mut PgConnection` as first parameter to all methods
- [ ] 2.3 Run `cargo check` and fix any compilation errors in domain layer

## 3. Infrastructure Layer — Update PgBookRepository and PgAuthorRepository

- [ ] 3.1 Update `PgBookRepository` to use injected `&mut PgConnection` instead of `self.pool.begin()` in `create`, `update`, `delete`, `restore`
- [ ] 3.2 Update `PgAuthorRepository` to use injected `&mut PgConnection` instead of `self.pool.begin()` in `create`, `update`, `delete`, `restore`
- [ ] 3.3 Remove `PgImportBooksRepository` and delete `src/infrastructure/import_books_repository.rs`
- [ ] 3.4 Remove `ImportBooksRepository` trait and delete `src/domain/repository/import_books_repository.rs`
- [ ] 3.5 Run `cargo check` and fix compilation errors in infrastructure layer

## 4. Use-Case Layer — Add Transaction Control for Book and Author Interactors

- [ ] 4.1 Update `CreateBookInteractor` to accept `pool: PgPool`, wrap `book_repository.create` in `pool.begin() … tx.commit()`
- [ ] 4.2 Update `UpdateBookInteractor` to accept `pool: PgPool`, wrap `find_by_id` + `update` in a transaction
- [ ] 4.3 Update `DeleteBookInteractor` to accept `pool: PgPool`, wrap `delete` in a transaction
- [ ] 4.4 Update `CreateAuthorInteractor` to accept `pool: PgPool`, wrap `create` in a transaction
- [ ] 4.5 Update `UpdateAuthorInteractor` to accept `pool: PgPool`, wrap `find_by_id` + `update` in a transaction
- [ ] 4.6 Update `DeleteAuthorInteractor` to accept `pool: PgPool`, wrap `delete` in a transaction
- [ ] 4.7 Update `RestoreBookInteractor` to accept `pool: PgPool`, wrap `restore` in a transaction
- [ ] 4.8 Update `RestoreAuthorInteractor` to accept `pool: PgPool`, wrap `restore` in a transaction
- [ ] 4.9 Rewrite `ImportBooksInteractor` to use `BookRepository` + `AuthorRepository` + `PgPool`, removing `ImportBooksRepository` dependency; manage its own transaction
- [ ] 4.10 Update `MutationInteractor` constructor to accept and pass `PgPool` for book/author interactors
- [ ] 4.11 Run `cargo check` and fix compilation errors in use-case layer

## 5. Dependency Injection Layer

- [ ] 5.1 Update `src/dependency_injection.rs` to pass `pool` into book/author interactors that now require it
- [ ] 5.2 Remove `PgImportBooksRepository` instantiation and wiring
- [ ] 5.3 Update type aliases (`QI`, `MI`, etc.) if generic parameter counts change
- [ ] 5.4 Run `cargo check` and fix DI layer compilation errors

## 6. Unit Tests

- [ ] 6.1 Update `src/use_case/interactor/book.rs` tests: add `always()` matcher for `&mut PgConnection` in all `MockBookRepository` expectations
- [ ] 6.2 Update `src/use_case/interactor/author.rs` tests: add `always()` matcher for `&mut PgConnection` in all `MockAuthorRepository` expectations
- [ ] 6.3 Update `src/use_case/interactor/mutation.rs` tests: adjust for new `PgPool` parameter in interactors
- [ ] 6.4 Rewrite `ImportBooksInteractor` unit tests to use `MockBookRepository` + `MockAuthorRepository` instead of `MockImportBooksRepository`
- [ ] 6.5 Run `cargo test --lib` and ensure all unit tests pass

## 7. Database Integration Tests

- [ ] 7.1 Update `PgBookRepository` `#[sqlx::test]` tests to pass `&mut conn` (obtained from `pool.begin()` or `pool.acquire()`) to repository methods
- [ ] 7.2 Update `PgAuthorRepository` `#[sqlx::test]` tests similarly
- [ ] 7.3 Remove `PgImportBooksRepository` `#[sqlx::test]` tests
- [ ] 7.4 Run `cargo test --features test-with-database` and ensure all DB tests pass

## 8. E2E Tests

- [ ] 8.1 Run E2E test suite against Docker Compose stack
- [ ] 8.2 Fix any failures caused by behavioral changes (none expected since this is an internal refactor with no API changes)

## 9. Code Quality & Final Verification

- [ ] 9.1 Run `cargo fmt --check` and fix formatting
- [ ] 9.2 Run `cargo clippy --fix --all-targets -- -D warnings` and fix all warnings
- [ ] 9.3 Run `cargo test` (full suite) and ensure 100% pass rate
- [ ] 9.4 Review `AGENTS.md` for consistency with the new architecture

## Future Phases (Out of Scope for This Change)

### Phase 2 — Remaining Repositories
Apply `&mut PgConnection` pattern to:
- `UserRepository` and `PgUserRepository`
- `BookEventRepository` and `PgBookEventRepository`
- `AuthorEventRepository` and `PgAuthorEventRepository`
Move transaction boundaries from these infrastructure repositories into their use-case interactors. Update DI layer accordingly.

### Phase 3 — Cleanup & Standardization
- Decide whether `QueryInteractor` should receive `PgPool` for transactional reads or keep `&self` repository calls.
- Introduce a thin `Connection` abstraction if direct `sqlx` dependency in domain layer becomes problematic.
- Audit all remaining `pool.begin()` calls in infrastructure to ensure none are left behind.
- Re-evaluate `AGENTS.md` rules after full migration is complete.
