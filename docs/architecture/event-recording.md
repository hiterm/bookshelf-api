# Event Recording

This document describes the *current* architecture for recording entity
change events. It is a description of how the code is built today, not an
immutable rule — it may change as the codebase evolves. It complements
`docs/database.md`, which documents the event-log database schema.

## Invariant

Every `create`, `update`, `delete`, and `restore` operation on any entity
records an event inside the same database transaction. This applies to
existing entities (`Book`, `Author`) and any new entity added in the future.

## Transaction boundary (use-case layer)

The transaction boundary is owned by the use-case layer via the
`TransactionManager` domain trait: an interactor calls
`transaction_manager.begin(user_id, operation)` to open a transaction,
passes the resulting transaction by `&mut` into each mutating repository method,
and calls `transaction_manager.commit(tx)` at the end. Mutating repositories get
the user from the transaction opened by `begin`; callers do not pass a second
`user_id` to those methods. This lets a single interactor compose multiple
repositories (e.g. the bulk import composes `BookRepository` and
`AuthorRepository`) inside one transaction.

The use-case layer knows two event concepts: the `EventSetOperation` passed
to `begin`, and the generated `event_set.id` exposed by the transaction so
mutation results can return an `eventSetId` after a successful commit. Event
row creation and persistence details remain in the infrastructure layer.

## Infrastructure responsibilities

- `PgTransactionManager::begin` generates the `event_set` UUID, binds the
  transaction to the user, exposes that id on the transaction, and inserts the
  single `event_set` row (the one place `event_set` rows are created).
- Each mutating `Pg*` repository method reads `tx.user_id()` for row ownership,
  reads `tx.event_set_id()` for event recording, and inserts the per-event
  `<entity>_event` rows. Domain repository traits expose only an associated
  `Transaction` type; they carry no other event knowledge.

## Adding a new entity or mutation operation

- Drive the operation inside a single transaction opened via
  `TransactionManager::begin` with the appropriate `EventSetOperation`.
- Mutating repository methods accept `tx: &mut Self::Transaction`, read the
  user from the transaction, and read `tx.event_set_id()`; they must not open
  their own transaction, accept a separate `user_id`, or create `event_set`
  rows.
- Create a dedicated `<entity>_event` table (and `<entity>_event_author`-style
  join tables if needed) following the `book_event` / `author_event` schema.
- The `event_set` row is inserted once in `PgTransactionManager::begin`; each
  operation inserts one row into the entity's event table.
- Add a new `EventSetOperation` variant (with its `as_str` round-trip) and the
  matching `event_set_operation` value via migration (e.g. `create_foo`,
  `update_foo`).

## References

- `docs/database.md` — event-log database schema.
- `.agent/plans/20260429-add-change-history.md` — the full design and the
  Decision Log for rationale.
- `.agent/plans/20260612-remove-import-books-repository.md` — the move of the
  transaction boundary into the use-case layer.
