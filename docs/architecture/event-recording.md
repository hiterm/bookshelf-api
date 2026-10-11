# Operation and Revision Recording

This document describes the current mutation-history architecture.
Operation/Revision is the authoritative write model and the only GraphQL
history contract. The legacy Event/EventSet code and tables have been removed.

The general dependency rules for domain, use-case, infrastructure, and
presentation code are defined in [`layers.md`](layers.md).

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

Book create/update accept existing author IDs and optional new author names.
New authors, the book, relationships and every history row share the same
CreateBook/UpdateBook operation and transaction. An author-name conflict aborts
the whole save; import alone retains find-or-create semantics. Existing authors
are not revised merely because a book references them. The response revision
number always identifies the resulting book revision, even when authors were
created alongside it.

Single-entity create/update repositories return a validated `RevisionNumber`.
Restore returns the entity together with its newly appended revision number.
Use cases build response metadata directly from these explicit results after
commit. Transactions retain user/Operation context without a last-revision
slot; bulk mutations have no last-revision result.

## Undo command boundary

`HistoryQueryInteractor` and `HistoryRepository` serve history reads only.
`HistoryCommandInteractor` validates the authenticated owner and target ID,
then invokes `use_case::port::undo::UndoCommand`. This port guarantees one
complete atomic Undo; it returns the new Operation ID only after commit.
Dependency injection wires mutations to `PgUndoCommand`, separately from
history queries and loaders.

The PostgreSQL implementation owns the database mechanics: begin a
SERIALIZABLE transaction, lock the owned target Operation, acquire ordered
entity locks, revalidate the current state, apply inverse state, append fresh
Revisions and OperationChanges, and commit. Any error rolls back the whole
command. `is_operation_undoable` is an advisory read; the command never trusts
an earlier eligibility result and always revalidates after locking.

Eligibility compares current existence and latest revision numbers with every
change's after state. It does not predict missing source Authors or restored
Author-name collisions; execution checks those constraints and rolls back on
failure. Advice can also become stale before execution. PostgreSQL contract
tests cover both entity types, tenant isolation, rollback, and contending Undo
commands with exactly one committed inverse.

Ordinary restore and Undo share owned source-to-current writes in
`infrastructure::revision_restoration`. Caller-specific decoding and dependency
errors remain at their original boundaries. Ordinary restore supplies the
returned entity's update timestamp; Undo uses the SERIALIZABLE transaction's
current timestamp. Both retain the source creation timestamp and Author links.

`infrastructure::history_recording` allocates the next revision with checked
integer overflow, copies the persisted current row and Book-Author links into
fresh history, and records the before/after change. Ordinary transaction
wrappers and Undo call the same connection-based helpers. Helpers never begin
or commit transactions; Undo retains its isolation, locks, revalidation, and
Author-before-Book restoration / Book-before-Author deletion order.

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

## References

- `docs/database.md` — database tables and constraints
- `openspec/changes/redesign-history-model/design.md` — redesign decisions
