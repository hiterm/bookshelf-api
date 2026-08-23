## 1. Bulk Author Persistence

- [x] 1.1 Add an import-oriented bulk author resolution method to the domain repository trait and mocks while preserving the single-author method
- [x] 1.2 Implement conflict-aware bulk author insertion, one bulk ID lookup, and bulk new-author event insertion in `PgAuthorRepository`
- [x] 1.3 Add unit and database-backed tests for new, existing, mixed, duplicate, event snapshot, shared event-set, and rollback author cases

## 2. Bulk Book Persistence

- [x] 2.1 Add a bulk book creation method to the domain repository trait and mocks while preserving the single-book method
- [x] 2.2 Implement bulk book, book-author, book-event, and book-event-author insertion in `PgBookRepository`
- [x] 2.3 Add database-backed tests for varied author counts, relationship and snapshot correctness, shared event sets, 1,000 books, and rollback

## 3. Import Orchestration

- [x] 3.1 Refactor `BookCommandInteractor::import` to deduplicate author names, invoke each bulk repository once, map IDs without duplicates, and preserve one transaction and commit
- [x] 3.2 Update interactor unit tests for validation-before-transaction, deduplication, bulk call counts and arguments, operation selection, commit behavior, and error propagation

## 4. Verification

- [x] 4.1 Confirm the import path has no awaited SQL or repository calls in book-count or author-count loops and needs no event-recording documentation change
- [x] 4.2 Run formatting, clippy, unit tests, database-feature tests, and the existing import E2E without modifying its scenarios
- [x] 4.3 Review the final diff, verify all OpenSpec tasks and artifacts, and record the final change status
