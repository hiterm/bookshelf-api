## Why

Clients currently must execute `importBooks` to learn whether an import will
succeed and which authors will be reused or created. A faithful preview is
needed that exercises the same database-backed import path without leaving any
persistent state.

## What Changes

- Add a `previewBookImport` GraphQL mutation accepting the existing
  `ImportBookInput` list and returning preview books with per-author
  `EXISTING` or `NEW` status.
- Execute previews through the same validation, bulk author resolution, book
  persistence, constraint, and event-recording path as `importBooks`, then
  explicitly roll back the transaction.
- Add explicit rollback support to the transaction abstraction and report
  which authors a bulk find-or-create operation created without adding
  per-author queries.
- Preserve the existing `importBooks` input, output, event-recording, atomicity,
  and batch-limit behavior.
- Exclude generated book, author, and event identifiers and timestamps from the
  preview contract because the corresponding rows do not survive rollback.
- Document that preview results are advisory: a later import re-evaluates the
  current database state and may observe concurrent changes.

## Capabilities

### New Capabilities
- `book-import-preview`: Faithful rollback-based preview of bulk book imports,
  including per-book author creation status and a no-persistent-state guarantee.

### Modified Capabilities
- `bulk-book-import`: Extend the bulk author-resolution contract to expose
  created-versus-existing status while preserving bounded queries, and require
  preview and import to share the same execution path.

## Impact

- GraphQL mutation schema and presentation objects for book import.
- Book command use-case DTOs, transaction lifecycle, and shared import
  execution logic.
- `TransactionManager` and its PostgreSQL adapter gain explicit rollback.
- Author repository bulk-resolution results include creation status.
- Unit, repository/integration, schema, and GraphQL E2E coverage expand; no new
  external dependency or transaction-external side effect is introduced.
