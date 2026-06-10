## 1. Fix import_books event_set operation

- [ ] 1.1 Add `event_set` INSERT with `import_books` operation in `ImportBooksInteractor::import`
- [ ] 1.2 Add `#[sqlx::test]` integration tests for `ImportBooksInteractor` covering:
  - `import_books` event_set operation is recorded
  - New authors create events, existing authors reuse without duplicate events
  - Shared author names deduplicate across books
  - Transaction rollback on failure (empty title, etc.)
  - Empty author names handled correctly
- [ ] 1.3 Run `cargo test --features test-with-database` to verify existing tests pass

## 2. Add eventSet GraphQL query

- [ ] 2.1 Add `EventSetDto` to `src/use_case/dto/event.rs`
- [ ] 2.2 Add `find_event_set_by_id` method to `QueryUseCase` trait in `src/use_case/traits/query.rs`
- [ ] 2.3 Implement `find_event_set_by_id` in `QueryInteractor` in `src/use_case/interactor/query.rs`
- [ ] 2.4 Add `EventSet` GraphQL object to `src/presentation/graphql/object.rs`
- [ ] 2.5 Add `eventSet` resolver to `src/presentation/graphql/query.rs`
- [ ] 2.6 Add unit tests for `QueryInteractor::find_event_set_by_id` in `src/use_case/interactor/query.rs`
- [ ] 2.7 Run `cargo test` to verify unit tests pass

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
