## 1. Unit of Work Core Implementation

- [x] 1.1 Create `PgUnitOfWork` struct in `src/infrastructure/unit_of_work.rs` with `begin`, `commit`, `rollback`, and `tx` methods
- [x] 1.2 Add `unit_of_work` module to `src/infrastructure.rs` (or `mod.rs`)
- [x] 1.3 Write unit tests for `PgUnitOfWork` lifecycle (begin, commit, rollback)
- [x] 1.4 Add `#[cfg(feature = "test-with-database")]` integration tests for `PgUnitOfWork` commit and rollback scenarios

## 2. Repository Refactoring — Core Methods

- [x] 2.1 Refactor `PgBookRepository`: extract `create_core`, `update_core`, `delete_core`, `restore_core` with `pub(in crate::infrastructure)` visibility
- [x] 2.2 Refactor `PgAuthorRepository`: extract `create_core`, `update_core`, `delete_core`, `restore_core` with `pub(in crate::infrastructure)` visibility
- [x] 2.3 Refactor `PgUserRepository`: extract `create_core` (if applicable) with `pub(in crate::infrastructure)` visibility
- [x] 2.4 Ensure existing pool-based public methods delegate to `*_core` and handle `begin/commit`
- [x] 2.5 Verify existing `#[cfg(feature = "test-with-database")]` tests still pass after refactoring

## 3. Import Books Service

- [x] 3.1 Define `ImportBooksService` trait in `src/domain/service/import_books_service.rs` (or `src/use_case/traits/import_books.rs` if domain service layer is not preferred)
- [x] 3.2 Create `PgImportBooksService` in `src/infrastructure/import_books_service.rs` using `PgUnitOfWork`, `PgBookRepository`, and `PgAuthorRepository`
- [x] 3.3 Implement `import` method with atomic author upsert → book creation → event recording within a single UoW
- [x] 3.4 Add `#[cfg(feature = "test-with-database")]` integration tests for `PgImportBooksService`

## 4. Remove Legacy Import Books Repository

- [x] 4.1 Delete `src/domain/repository/import_books_repository.rs` and remove from `src/domain/repository.rs`
- [x] 4.2 Delete `src/infrastructure/import_books_repository.rs`
- [x] 4.3 Remove `ImportBooksRepository` from all `use` statements and module declarations
- [x] 4.4 Update `ImportBooksInteractor` to depend on `ImportBooksService` trait instead of `ImportBooksRepository`
- [x] 4.5 Update `ImportBooksInteractor` unit tests to use `MockImportBooksService` instead of `MockImportBooksRepository`

## 5. Update Dependency Injection and Application Wiring

- [x] 5.1 Update the application composition root (e.g., `main.rs` or DI setup) to wire `PgImportBooksService` instead of `PgImportBooksRepository`
- [x] 5.2 Ensure `MutationInteractor` and other top-level interactors receive the correct dependencies

## 6. Verification and Cleanup

- [x] 6.1 Run `cargo fmt` and fix any formatting issues
- [x] 6.2 Run `cargo clippy --fix --all-targets -- -D warnings` and resolve all warnings
- [x] 6.3 Run `cargo test` (unit tests without database feature)
- [x] 6.4 Run `cargo test --features test-with-database` (integration tests with database)
- [x] 6.5 Run `cargo test` for E2E tests if applicable
- [x] 6.6 Review and update `AGENTS.md` if any guidelines or conventions have changed
