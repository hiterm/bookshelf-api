## Why

`mergeAuthor` currently persists each affected Book through a separate
`BookRepository::update()` call. Because each update writes the Book, its author
relationships, and its event snapshot, SQL round trips grow with the number of
Books linked to the source Author.

## What Changes

- Add a general-purpose `BookRepository::update_all()` operation that persists
  multiple already-updated Book entities with set-based SQL.
- Change `mergeAuthor` to continue applying `Book::update()` to every affected
  Book, then persist all Books with one repository call.
- Bulk-update Book rows, synchronize `book_author`, create one update event per
  Book, and record each event's final author snapshot.
- Preserve the existing merge behavior, lock order, event semantics, GraphQL
  contract, and database uniqueness constraints.
- Define an empty bulk update as a successful no-op.

## Capabilities

### New Capabilities

- `bulk-book-update`: Persist multiple updated Book aggregates and their update
  events using a number of SQL round trips that does not grow with the Book
  count.

### Modified Capabilities

- None.

## Impact

- Domain repository contract: `src/domain/repository/book_repository.rs`
- PostgreSQL persistence: `src/infrastructure/book_repository.rs`
- Author merge orchestration: `src/use_case/interactor/author.rs`
- Repository, use-case, database integration, and regression tests
- No GraphQL schema, response, dependency, or database schema change
