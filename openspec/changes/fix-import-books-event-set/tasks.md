## 1. Add eventSet GraphQL query (TDD)

- [ ] 1.1 Add unit tests for `QueryInteractor::find_event_set_by_id` (test-first)
- [ ] 1.2 Run `cargo test` to confirm tests fail (missing `EventSetDto`, resolver, etc.)
- [x] 1.3 Add `EventSetDto` to `src/use_case/dto/event.rs`
- [x] 1.4 Add `find_event_set_by_id` method to `QueryUseCase` trait in `src/use_case/traits/query.rs`
- [x] 1.5 Implement `find_event_set_by_id` in `QueryInteractor` in `src/use_case/interactor/query.rs`
- [x] 1.6 Add `EventSet` GraphQL object to `src/presentation/graphql/object.rs`
- [x] 1.7 Add `eventSet` resolver to `src/presentation/graphql/query.rs`
- [x] 1.8 Run `cargo test` to verify all tests pass

## 2. Fix import_books event_set operation (TDD)

- [ ] 2.1 Convert existing `ImportBooksInteractor` unit tests to `#[sqlx::test]` integration tests (test-first)
- [ ] 2.2 Run `cargo test --features test-with-database` to confirm tests fail (event_set table missing or no import_books INSERT)
- [ ] 2.3 Add `event_set` INSERT with `import_books` operation in `ImportBooksInteractor::import`
- [ ] 2.4 Run `cargo test --features test-with-database` to verify all tests pass

## 3. Update E2E tests

- [ ] 3.1 Add `event_set.operation` validation to `e2e_import_books` test
- [ ] 3.2 Add `event_set.operation` validation to `e2e_book_events_records_create_operation` test
- [ ] 3.3 Add `event_set.operation` validation to `e2e_book_events_records_update_operation` test
- [ ] 3.4 Add `event_set.operation` validation to `e2e_author_events_records_create_operation` test
- [ ] 3.5 Add `event_set.operation` validation to `e2e_author_events_records_update_operation` test
- [ ] 3.6 Run E2E tests to verify all pass

## 4. Pre-commit verification

- [ ] 4.1 Run `cargo fmt --check`
- [ ] 4.2 Run `cargo clippy --fix --all-targets -- -D warnings`
- [ ] 4.3 Run `cargo test --features test-with-database`
- [ ] 4.4 Run E2E tests
