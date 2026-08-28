# Operation and Revision Recording

This document describes the current mutation-history architecture. Legacy
Operation/Revision is the authoritative write model and API contract. Legacy
Event/EventSet tables and read paths remain temporarily until the cleanup PR,
but no mutation writes them.

## Invariant

Every Book or Author create, update, delete, restore, import, or merge creates
exactly one owned `operation` and records every affected entity in the same
database transaction. A state that exists is represented by an immutable full
Revision. Absence is represented only by a nullable side of an
OperationChange:

- create: `none -> revision 1`
- update or restore: `revision N -> revision N+1`
- delete: `revision N -> none`

No current-state mutation may commit without its Operation, Revision where
applicable, and OperationChange.

## Transaction boundary

The use-case layer opens one transaction through
`TransactionManager::begin_operation(user_id, NewOperation)`. The transaction
carries the authenticated `user_id` and generated `operation_id`; repositories
derive both values from it. The use case composes repositories and commits only
after all current-state and history writes succeed. Preview explicitly rolls
the same path back.

`NewOperation` supplies a validated type and optional typed detail. Import
records its item count, merge records source and destination Author IDs, and
restore records the source revision number. Mutation responses expose the
Operation ID and, for a single resulting entity revision, its revision number.

## Tenant-aware identity

GraphQL identifies a revision as `(entityId, revisionNumber)`. The server adds
`user_id` from authentication. Database identity and foreign keys include that
owner:

- Book Revision: `(user_id, book_id, revision_number)`
- Author Revision: `(user_id, author_id, revision_number)`
- Operation ownership: `(operation_id, user_id)`

OperationChange rows carry `user_id` and reference both Operation and Revision
through composite foreign keys. The database therefore rejects cross-tenant
history links even when two users own entities with the same UUID.

Revision numbers are allocated independently for each `(user_id, entity_id)`.
Single-entity updates lock the owned current row before allocation. Restoring a
deleted entity locks an owned source revision as a stable serialization key.
Bulk paths use deterministic ordering and set-based statements.

## Restore

`restoreBook(bookId, revisionNumber)` and `restoreAuthor(authorId,
revisionNumber)` load only an owned source revision. Restore preserves the
source lifecycle creation time, refreshes the lifecycle update time, writes the
current row, and appends a fresh revision; it never makes an old revision
current by identity.

A Book restore verifies that every Author referenced by the source revision is
currently present for the same user. Missing or cross-tenant references abort
the complete transaction.

## Reads

`operations` hides baseline operations and returns newest first;
`operation(id)` is ownership scoped. Book and Author revision lists are newest
first, while exact revision lookup supports restore and nested change
resolution. Nested Book and Author changes are loaded only when selected and
batch by Operation IDs.

## Temporary legacy read paths

Until the cleanup PR, legacy Event/EventSet tables, code, and GraphQL read paths
remain available only for history recorded before this change. They are
read-only residuals: new mutations write only Operation/Revision history, and
the GraphQL mutation contract exposes no Event identifiers or aliases. PR 3
removes the residual code, API, and tables completely.

## References

- `docs/database.md` — database tables and constraints
- `openspec/changes/redesign-history-model/design.md` — redesign decisions
