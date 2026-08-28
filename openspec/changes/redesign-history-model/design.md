## Context

The application currently stores live Book and Author rows plus append-only Event snapshots grouped by EventSet. Event and EventSet simultaneously describe mutation mechanics, snapshots, restore sources, and logical operations. That coupling makes a multi-entity operation hard to inspect as one semantic action and makes a general inverse operation unsafe to define.

The existing use-case layer owns one PostgreSQL transaction per logical operation through `TransactionManager`; repositories mutate current state and record history in that transaction. This useful boundary remains. The primary GraphQL client is controlled with the API, so the final contract can make a clean break instead of maintaining Event aliases. Existing Event history need not survive, but every live Book and Author at deployment must start with a baseline revision.

Implementation starts from validated `main` commit `6c62075bb6cfb49612d68a4575082ec4e2ed8a9d`. The current mutation path is `use_case/interactor/{book,author}.rs` → `domain/repository/{book,author,transaction}.rs` → `infrastructure/{book,author,transaction}.rs`; history queries run through `use_case/{traits,interactor}/event.rs`, the three Event repositories/adapters, and GraphQL query/object/loader modules. The existing schema originates in `migrations/20260429040611_add_event_tables.sql` with merge additions in `20260820000000_add_merge_author_operations.sql`; migration coverage is in `migrations/test/test_migration.mjs`, generated API shape is `schema.graphql`, and history/import/restore coverage is under `e2e/tests/graphql_*.rs`.

## Goals / Non-Goals

**Goals:**

- Separate logical Operations, complete per-entity Revisions, and before/after OperationChanges.
- Keep `book`, `book_author`, and `author` as the authoritative current state.
- Make revision creation inseparable from repository mutation in one transaction.
- Support revision inspection, restore, and conflict-safe atomic undo across multiple entities.
- Preserve ownership isolation, set-based import/merge paths, lazy nested GraphQL loading, and preview rollback.
- Reach a final state with all legacy Event/EventSet code, APIs, specs, and tables removed.

**Non-Goals:**

- Event Sourcing or reconstructing current state from history.
- Migrating legacy Event/EventSet history.
- A dedicated redo API; undo-of-undo remains structurally possible.
- Revision garbage collection or retention limits in the initial implementation.
- Restoring the historical state of referenced Authors while restoring a Book.

## Decisions

### Separate Operation identity from entity revision identity

`operation` has a UUID, owner, type, nullable JSONB detail, optional `undo_of_operation_id`, and database-managed creation time. Operation type is represented as a closed Rust enum; its detail is converted at the boundary to/from typed per-operation structures. Operation rows never contain entity snapshots.

Within an authenticated tenant, Book and Author revisions use `(entity_id, revision_number)` as their API/domain identity. Because current entity primary keys permit the same UUID for different users, database primary and foreign keys additionally include `user_id`: `(user_id, entity_id, revision_number)`. Revision numbers are positive, start at 1 independently per owned entity, and increase monotonically. Avoiding a revision UUID makes the public restore source and ordering explicit. A pointer on the live entity is omitted initially; the latest revision is found by maximum revision number under an entity/owner lock.

Alternative considered: keep Event IDs as revision identity. Rejected because a global surrogate conflates audit ordering with an entity's own version sequence and perpetuates the old model.

### Store complete append-only snapshots

`book_revision` stores every Book scalar and lifecycle timestamp. `book_revision_author` stores the Author IDs in that Book aggregate revision. `author_revision` stores every Author scalar and lifecycle timestamp. Revision rows are never updated; restore copies a historical snapshot to current state and appends a new revision representing that new current state.

Alternative considered: store diffs. Rejected because reads, restore, undo, schema evolution, and retention become more complex and error-prone for modest entity sizes.

### Represent absence only in OperationChange

`book_operation_change` and `author_operation_change` contain the operation, entity ID, nullable before revision number, and nullable after revision number. Create is `none -> rev1`, update/restore is `revN -> revN+1`, and delete is `revN -> none`. Both sides cannot be null. Nullable composite foreign keys reference the matching entity revision. Owner consistency is a mandatory invariant: database constraints include `user_id` in composite Operation and Revision foreign keys, and every repository operation is tenant-scoped.

No deleted revision is created. The last full revision remains the restore source, while absence is an operation transition rather than an invented entity state.

### Preserve the use-case transaction boundary and move operation context into it

`TransactionManager::begin(user_id, operation_type)` starts one database transaction, inserts exactly one Operation, and stores `operation_id` and `user_id` in `PgTransaction`. The use case chooses semantic type/detail and composes repositories. Each mutating repository reads the context and atomically changes current rows, appends revisions, and inserts OperationChanges. A failed mutation, history insert, preview, or commit leaves none of those rows behind.

Alternative considered: let use cases explicitly write revisions after repository mutations. Rejected because a new call site could update current state while forgetting history.

### Serialize revision allocation per entity

Updates and deletes lock the owned current entity before reading the maximum revision. Creates introduce revision 1 with the new entity. Restoring a deleted entity locks an ownership-scoped stable database key/history row before allocating the next number. Bulk paths use deterministic Book-before-Author and entity-ID ordering plus set-based statements to retain bounded round trips and avoid deadlocks.

Database primary/unique constraints reject duplicate revision numbers as a final guard. Repository integration tests cover concurrent or conflicting allocation where the test harness supports it.

### Seed baseline history from current state

The first migration creates new tables, then creates one internal `baseline` Operation per user that currently owns at least one Book or Author. It inserts revision 1 for every current entity, snapshots current Book-Author references, and records `none -> rev1` changes. Legacy Event rows are not copied. Baseline Operations remain queryable internally but normal operation lists exclude them by default so they do not appear as user actions.

The migration is transactional. Rolling back the deployment migration removes the new schema and baseline rows without touching current or legacy state.

### Restore validates current references and always creates history

`restoreBook(bookId, revisionNumber)` and its Author counterpart load an owned source revision. A Book restore requires every recorded Author ID to exist currently for the same owner; it does not restore Author snapshots. Whether the Book currently exists or is deleted, the repository copies the source data, applies the restore operation lifecycle timestamp, appends the next Book revision, and records the corresponding before/after change. Missing or cross-tenant sources/references fail the entire transaction.

### Undo eligibility is entity-local and revalidated under lock

An Operation is undoable only when it is owned, is not baseline, and every change's after-state matches current state. A non-null after revision must be the entity's current maximum revision and the entity must exist; a null after revision requires the entity to remain absent. Later Operations affecting unrelated entities do not matter.

`undoOperation` begins a new `undo` Operation linked through `undo_of_operation_id`, locks every affected entity/history key in deterministic order, and repeats eligibility and reference validation. It then applies each target before-state. For related multi-entity Operations, restorations run Author before Book so restored Books can resolve their Authors, while deletions run Book before Author so no live Book retains a deleted Author reference. Restoring content appends a fresh revision; undoing a create deletes the entity and records `current -> none`. PR 2 import and merge E2E tests will verify this ordering and that all changes commit or roll back together. Undoing an undo is not specially exposed but the model does not prohibit it.

### Replace the GraphQL history contract directly

Queries expose `operations`, `operation(id)`, `bookRevisions(bookId)`, `bookRevision(bookId, revisionNumber)`, and Author equivalents. Operation nested Book/Author changes batch by Operation IDs and load only when selected. Changes expose before/after Revision objects. All paths are tenant-scoped.

Mutation payloads replace `eventSetId` with `operationId`; create/update/restore results that have one affected revision expose `revisionNumber`. Restore arguments use entity ID plus revision number. `undoOperation(operationId)` returns its new Operation ID and affected result metadata. This breaking contract applies from PR 1 with no compatibility aliases.

### Deliver in three implementation stages

PR 1 adds the new schema, baseline, transaction context, revision recording, restore, queries, payloads, and tests while legacy tables and internal code may coexist as read-only residuals. PR 1 removes the Event/EventSet GraphQL surface completely; new mutations write only the new model, and Operation/Revision is the sole API contract. PR 2 adds undo and its full test matrix. PR 3 removes the remaining internal Event/EventSet code, database objects, stale names, and old specs, then updates architecture documentation. The OpenSpec change describes this final state and is archived only after all three PRs merge.

## Risks / Trade-offs

- [Concurrent revision allocation could collide] → Lock owned entities/history keys deterministically and retain composite uniqueness constraints.
- [Revision tables grow without bound] → Add query indexes and keep append-only identity compatible with future retention/GC; GC policy is deferred.
- [JSON operation detail loses compile-time guarantees] → Use typed Rust detail variants and validate serialized shape at repository boundaries.
- [Baseline migration can be expensive] → Use set-based `INSERT ... SELECT`, transactionally, with indexes created in an order that avoids per-row application work.
- [Temporary legacy read paths increase complexity] → Make Operation/Revision the only write model and mutation contract in PR 1, then remove the read-only residuals after undo is green.
- [Undo can violate current references] → Revalidate all entities and Book Author references under the same transaction immediately before writes.
- [Breaking GraphQL changes disrupt clients] → Accept the PR 1 break explicitly and migrate the frontend separately without API aliases.

## Migration Plan

1. Record the latest validated `main` SHA and create the single OpenSpec change.
2. PR 1 creates new tables and baseline rows, migrates runtime writes and reads, updates GraphQL contracts/docs/tests, validates migration from the existing schema, and merges only after CI and CodeRabbit approval.
3. PR 2 starts from updated `main`, implements/revalidates atomic undo, adds unit, database, and E2E coverage, and merges after the same gates.
4. PR 3 starts from updated `main`, removes legacy APIs/code/tables in dependency order, updates docs and schema snapshots, and merges after the same gates.
5. A final archive PR syncs delta specs into `openspec/specs`, validates the archive and CI, and merges without requiring CodeRabbit.

Rollback before PR 3 can return application traffic to the legacy release while the additive schema remains unused. PR 3's destructive table migration is intentionally last; rollback after it requires restoring the database from backup or accepting loss of legacy history, which is allowed by the product requirement. Current Book/Author data is never derived from or deleted with legacy history.

## Open Questions

- Whether normal `operations` queries expose an explicit `includeBaseline` argument or always hide baseline while direct lookup remains available; choose the smallest API consistent with frontend needs during PR 1.
- Exact pagination shape for Operation and Revision lists; reuse the repository's prevailing list conventions unless scale testing requires cursors.
- Whether `undoable` is exposed as a computed GraphQL field in PR 2 or clients attempt undo and handle a conflict; server-side execution validation is mandatory either way.
