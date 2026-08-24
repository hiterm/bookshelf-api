## Context

`importBooks` currently validates and maps the complete batch before opening a
write transaction, then bulk-resolves authors, builds books, bulk-persists books
and events, and commits one `ImportBooks` event set. PostgreSQL transactions
roll back on drop, but `TransactionManager` exposes only `begin` and `commit`.
The bulk author repository returns only name-to-ID mappings, so callers cannot
tell which inserts won `ON CONFLICT DO NOTHING`.

The preview must be as faithful as an import: it must execute repository writes,
event recording, and database constraints. It must also preserve the existing
rule that invalid input does not begin a transaction and must not expose IDs for
rows that are deliberately rolled back.

## Goals / Non-Goals

**Goals:**

- Add a GraphQL `previewBookImport` mutation using `ImportBookInput`.
- Share validation, author resolution, book construction, persistence, and
  event recording between preview and import.
- Explicitly roll back a successful preview and leave all import tables
  unchanged.
- Return only preview books and per-book author `EXISTING`/`NEW` status.
- Preserve bounded bulk SQL, validation semantics, the import contract, and
  event recording.

**Non-Goals:**

- Reserving preview results or guaranteeing a later import has identical author
  statuses.
- Adding `dryRun` to `importBooks`, a preview token, locks, or a preview-specific
  event-set operation.
- Returning book, author, event, event-set, or timestamp identifiers.
- Adding transaction-external notifications, files, messages, or other side
  effects.

## Decisions

### Expose preview as a dedicated Mutation

The schema adds `previewBookImport(books: [ImportBookInput!]!)` to Mutation.
Although successful execution leaves no final database state, it performs
inserts and event recording before rollback. Mutation makes that write-capable
execution explicit and keeps write dependencies in `BookCommandUseCase`.

Alternatives considered were a `dryRun` flag, which would complicate the
existing import contract, and Query, which would conceal a write transaction
behind read semantics.

### Use one prepared and transactional import execution path

Both operations use the same preparation/validation routine before `begin`,
preserving the existing no-transaction-on-validation-error behavior. Both then
invoke one private transactional `execute_import` routine that bulk-resolves
authors, removes duplicate author names per book, constructs books, and calls
`create_all`, including all author/book/event writes. The execution result owns
the generated domain books plus resolved author names and creation statuses;
presentation DTOs are derived afterward.

The only transaction-terminal difference is that import reads the event-set ID
and commits, while preview builds its ID-free DTO and rolls back. Copying the
repository path or using a preview-only lookup was rejected because either can
drift from import behavior and the latter adds queries.

### Return creation metadata from bulk author resolution

`find_or_create_by_names` returns resolved name/ID values plus the set of IDs
inserted by its existing bulk `INSERT ... RETURNING` statement. The subsequent
bulk lookup remains unchanged, so statement count stays bounded. Duplicate
input names are normalized before this repository method, as today.

A preliminary `find_by_names` was rejected because it adds a query, can race
with the insert, and does not describe what the actual find-or-create operation
created.

### Add explicit rollback at the transaction boundary

`TransactionManager::rollback` consumes its transaction, and
`PgTransactionManager` delegates to `sqlx::Transaction::rollback`. A successful
preview always calls it. If transactional execution fails, the uncommitted
transaction is dropped and sqlx performs rollback, preserving the original
execution error. If explicit rollback after successful execution fails, that
rollback/database error is returned because the preview cannot assert that its
cleanup completed. There is no second meaningful rollback error to combine
with an execution error under the consuming transaction API.

Concrete sqlx transaction operations remain outside the use-case layer.

### Treat preview output as advisory and ID-free

The response contains book fields and ordered, de-duplicated authors with
`EXISTING` or `NEW`. If one newly inserted author appears in several input
books, every occurrence is `NEW` because all refer to the creation performed by
that preview transaction. IDs and timestamps are omitted because their rows do
not survive rollback and generated values are execution-time details.

Preview and import are separate transactions. A concurrent write can change a
later import's author resolution; the import always re-executes against current
state. No reservation or locking is introduced.

### Require transaction-contained side effects

The shared execution path currently performs only writes through repositories
using the supplied transaction. This is a required invariant for rollback-based
preview: future external API calls, queue publishing, email, file writes,
out-of-transaction persistence, or irreversible audit operations must not be
added to it without redesigning preview semantics.

## Risks / Trade-offs

- **Preview performs real write work and can be expensive** → Keep the existing
  1,000-book limit and bounded bulk statements; clearly expose it as Mutation.
- **Preview can differ from a later import after concurrent changes** → Define
  the response as advisory and always re-run import against current state.
- **Rollback failure leaves outcome uncertain** → Return the database error and
  never report a successful preview.
- **Shared refactoring could alter existing import behavior** → Retain existing
  regression tests and add unit/E2E parity checks around validation,
  normalization, transaction boundaries, and event recording.

## Migration Plan

No schema migration is required. Deploy the additive GraphQL schema and server
implementation together. Older clients remain compatible because
`importBooks` is unchanged. Rollback is removing the new mutation and related
code; no persisted preview data needs cleanup.

## Open Questions

None.
