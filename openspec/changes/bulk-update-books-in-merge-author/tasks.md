## 1. Repository Contract and Persistence

- [x] 1.1 Add the general `BookRepository::update_all()` contract, including the successful empty-slice behavior
- [x] 1.2 Implement user-scoped set-based Book row updates with affected-row validation
- [x] 1.3 Implement set-based final `book_author` synchronization, including empty author sets and conflict-safe inserts
- [x] 1.4 Bulk-create one update event per Book and bulk-create final event-author snapshots

## 2. Author Merge Integration

- [x] 2.1 Update `AuthorCommandInteractor::merge()` to mutate every Book through `Book::update()` and call `update_all()` once without changing lock order
- [x] 2.2 Update merge use-case tests for one bulk call, final author sets, multiple Books, and the empty collection

## 3. Persistence and Regression Tests

- [x] 3.1 Add repository tests for multi-Book scalar updates, relationship add/remove/replace/conflict behavior, events, and empty input
- [x] 3.2 Add repository coverage for user isolation and transaction rollback on a bulk failure
- [x] 3.3 Add or extend real-database merge coverage for multiple Books, pre-existing destination relationships, Book snapshots, Author merge events, and one event set
- [x] 3.4 Assess E2E coverage and add only the minimum regression scenario if existing tests do not cover the critical multi-Book merge behavior

## 4. Validation and Delivery

- [x] 4.1 Verify `update_all()` contains no per-Book SQL execution and reconcile implementation details with the OpenSpec artifacts
- [x] 4.2 Run repository, use-case, integration, and E2E tests
- [x] 4.3 Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`
- [x] 4.4 Commit the completed change in logical increments, push the branch, and open a PR with the requested design and performance summary
- [ ] 4.5 Monitor PR CI, fix change-related failures, and confirm the PR is review-ready
