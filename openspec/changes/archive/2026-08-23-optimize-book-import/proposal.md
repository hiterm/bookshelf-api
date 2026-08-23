## Why

`importBooks` currently performs database writes and lookups once per book and
once per unique author, so SQL round trips grow as O(N + A) for N books and A
authors. The existing batch limit of 1,000 books makes this overhead material;
the import path should retain its behavior while using a nearly constant number
of SQL statements.

## What Changes

- Add import-oriented bulk author resolution that inserts missing authors,
  resolves all requested author IDs, and records new-author events in bulk.
- Add import-oriented bulk book creation that writes books, author
  relationships, book events, and event-author relationships in bulk.
- Deduplicate author names in memory and orchestrate each bulk repository call
  once from `BookCommandInteractor::import`.
- Preserve the GraphQL schema, validation and response semantics, 1,000-book
  limit, one-transaction boundary, and one shared import event set.
- Keep the existing single-author and single-book repository methods unchanged
  for non-import mutations.

## Capabilities

### New Capabilities
- `bulk-book-import`: Defines bounded-query bulk persistence and event-recording
  behavior for the existing `importBooks` mutation.

### Modified Capabilities

None.

## Impact

- Domain repository traits gain bulk methods dedicated to import orchestration.
- PostgreSQL author and book repositories gain array/`UNNEST`-based bulk SQL.
- The book command interactor changes from per-entity repository calls to one
  author bulk call and one book bulk call.
- Unit and database-backed repository coverage expands; the GraphQL API,
  database schema, dependencies, and existing E2E scenarios remain unchanged.
