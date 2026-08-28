## 1. PR 1 — Operation and Revision foundation

- [x] 1.1 Record validated starting `main` SHA and map existing Event mutation/query paths, migrations, generated schema, and test harnesses
- [x] 1.2 Add Operation, OperationType/detail, BookRevision, AuthorRevision, and Book/AuthorOperationChange domain models with unit tests
- [x] 1.3 Add migration tables, keys, ownership constraints, composite revision foreign keys, query indexes, and set-based per-user baseline Operations/Revisions/changes
- [x] 1.4 Add migration integration coverage for baseline Book scalars, Book Author references, Author state, ownership, empty users, and ignored legacy history
- [x] 1.5 Replace transaction `event_set_id` context with Operation ID and make `TransactionManager::begin` create the typed Operation
- [x] 1.6 Add repository interfaces and PostgreSQL adapters for owned Operation and Book/Author Revision/change queries, including batch lookup
- [x] 1.7 Migrate single Book create, update, and delete repositories to atomically allocate Revisions and record OperationChanges
- [x] 1.8 Migrate single Author create, update, delete, and find-or-create repositories to atomically allocate Revisions and record OperationChanges
- [x] 1.9 Migrate set-based import and preview paths to record all created Book/Author Revisions and changes under one ImportBooks Operation with complete rollback tests
- [x] 1.10 Migrate Author merge and bulk Book update to record source, destination, and every affected Book under one MergeAuthor Operation while preserving bounded queries and lock order
- [x] 1.11 Replace Event-based Book and Author restore with owned revision-based restore, fresh revision creation, lifecycle timestamp rules, and live Book Author-reference validation
- [x] 1.12 Add Operation and Revision use-case DTOs/interactors with unit tests for type/detail, ownership, revision ordering, exact lookup, and selection-independent batch methods
- [x] 1.13 Replace GraphQL Event mutation metadata and restore arguments with `operationId` and applicable `revisionNumber`
- [x] 1.14 Add `operations`, `operation`, Book Revision, and Author Revision GraphQL queries plus selection-driven batched nested changes and before/after Revision resolution
- [x] 1.15 Remove Event/EventSet GraphQL queries, types, loaders, mutation metadata, and restore arguments; update the generated schema, API E2E contracts, fixtures, and architecture/database docs for the breaking Operation/Revision-only contract; migrate the frontend separately
- [x] 1.16 Run PR 1 unit, database integration, migration, rollback, import, merge, restore, E2E, formatting, lint, full test, and OpenSpec validation suites
- [ ] 1.17 Commit granular PR 1 changes, push, create the PR, fix CI to green, request `@coderabbitai review`, address and reply to findings, re-request after fixes or rate-limit expiry, obtain approval, and merge

## 2. PR 2 — Operation undo

- [ ] 2.1 Update local `main` after PR 1 merge, record its SHA, and create a fresh `codex/` PR 2 branch
- [ ] 2.2 Implement owned undo eligibility for create, update, delete, restore, import, and merge changes using only each affected entity's current after-state
- [ ] 2.3 Add deterministic multi-entity locking and in-transaction revalidation for undo execution
- [ ] 2.4 Implement inverse application that restores Authors before Books, deletes Books before Authors, appends new Revisions for restored content, deletes creations without deleted Revisions, and records a linked Undo Operation with all changes
- [ ] 2.5 Validate current Book Author references during undo and roll back the complete multi-entity Operation on any failure
- [ ] 2.6 Add `undoOperation(operationId)` and decide/document whether Operation exposes computed `undoable`, while always revalidating server-side
- [ ] 2.7 Add unit and database tests for update, create, delete, restore, import, merge, related-entity restore/delete ordering, unrelated later Operations, conflicting later revisions, deleted-state matching, missing references, atomic rollback, and undo history
- [ ] 2.8 Add/update GraphQL E2E coverage and schema assets for undo, including import/merge ordering and atomicity plus future undo-of-undo representability
- [ ] 2.9 Run PR 2 formatting, lint, full tests, database integration, E2E, and OpenSpec validation
- [ ] 2.10 Commit granular PR 2 changes, push, create the PR, fix CI to green, complete CodeRabbit review/replies/re-review to approval, and merge

## 3. PR 3 — Legacy Event/EventSet cleanup

- [ ] 3.1 Update local `main` after PR 2 merge, record its SHA, and create a fresh `codex/` PR 3 branch
- [ ] 3.2 Remove Event/EventSet domain IDs, operations, entities, and conversion tests after confirming all runtime consumers use Operation/Revision
- [ ] 3.3 Remove Event/EventSet repository traits, PostgreSQL adapters, mocks, and repository tests
- [ ] 3.4 Remove Event query/DTO/interactor boundaries and legacy Event-based restore lookup
- [ ] 3.5 Remove remaining internal presentation fixtures and test terminology tied to Event/EventSet after confirming no public GraphQL contract remains
- [ ] 3.6 Add the final migration that drops legacy relationship, entity Event, EventSet, and lookup tables in foreign-key dependency order
- [ ] 3.7 Update migration-from-existing-schema tests and verify current state plus Operation/Revision history survive legacy table removal
- [ ] 3.8 Search the repository for legacy history names and remove unintended `event`, `event_set`, EventId, EventSetId, EventOperation, and EventSetOperation references
- [ ] 3.9 Replace current Event-recording architecture and database documentation with final Operation/Revision/Undo documentation
- [ ] 3.10 Run PR 3 formatting, lint, full tests, database integration, migration, GraphQL schema/E2E, and OpenSpec validation
- [ ] 3.11 Commit granular PR 3 changes, push, create the PR, fix CI to green, complete CodeRabbit review/replies/re-review to approval, and merge

## 4. OpenSpec archive PR

- [ ] 4.1 Update local `main` after PR 3 merge and verify every implementation task and completion condition
- [ ] 4.2 Run final OpenSpec validation and archive `redesign-history-model`, synchronizing delta specs and removing/replacing legacy Event specifications
- [ ] 4.3 Verify Operation, Revision, restore, undo, import/merge, payload, and cleanup requirements are correct in `openspec/specs`
- [ ] 4.4 Create the archive-only PR, confirm validation and CI green without CodeRabbit review, merge it, and verify the change is archived on `main`
