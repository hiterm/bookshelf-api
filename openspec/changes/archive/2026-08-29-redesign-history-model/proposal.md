## Why

The current Event/EventSet model mixes logical operations with entity snapshots, making multi-entity history, revision restore, and safe undo difficult to express. The history foundation must separate user-visible operations from append-only per-entity revisions while keeping the ordinary Book and Author tables authoritative.

## What Changes

- Introduce user-scoped Operations, complete Book and Author Revisions, and entity-specific OperationChanges that connect before and after revisions.
- Record each logical mutation and every affected entity revision atomically in the existing use-case-owned PostgreSQL transaction.
- Seed revision 1 and an internal baseline Operation from current Book and Author state without migrating legacy Event history.
- Replace Event-based restore with revision-based restore that always appends a new revision and validates live references.
- Add operation and revision queries with selection-driven, batched nested change loading.
- Add atomic Operation undo whose eligibility is based only on whether every affected entity still matches the target Operation's after-state.
- Preserve the real transactional write path and full rollback behavior for import preview.
- **BREAKING** Replace `eventId`/`eventSetId` mutation metadata and Event/EventSet GraphQL history APIs with `operationId`/`revisionNumber` and Operation/Revision APIs.
- **BREAKING** Remove the legacy Event/EventSet domain model, repositories, use cases, GraphQL contract, and database tables after Operation/Revision and undo are complete.
- Keep current-state tables authoritative; this change does not introduce Event Sourcing or migrate existing Event history.

## Capabilities

### New Capabilities

- `operation-history`: Logical Operation recording, typed details, per-entity changes, atomicity, ownership, and Operation queries.
- `entity-revision-history`: Append-only complete Book and Author snapshots, baseline creation, revision numbering, and revision queries.
- `revision-restore`: Restore current Book or Author state from an owned revision while appending a new revision and validating references.
- `operation-undo`: Determine Operation undo eligibility from affected entities and atomically record inverse changes as a new Operation.

### Modified Capabilities

- `book-import-preview`: Preview uses and fully rolls back Operation, Revision, and OperationChange writes.
- `bulk-book-import`: Imports record all created Books and Authors as revisions and changes under one Operation.
- `bulk-book-update`: Bulk Book updates record revisions and OperationChanges instead of legacy events.
- `canonical-mutation-payloads`: Mutation payloads expose Operation and revision metadata instead of Event identifiers.
- `entity-lifecycle-timestamps`: Revision and Operation audit timestamps replace Event audit timestamps; restore remains a new lifecycle update.
- `entity-mutation-event-ids`: Event-based mutation metadata and restore-source contracts are removed and replaced by revision metadata.
- `entity-use-case-boundaries`: History reads use Operation/Revision boundaries and mutation invariants record revisions rather than events.
- `event-set-query-model`: EventSet and entity Event history queries are removed in favor of Operation and Revision queries with equivalent lazy batched loading.

## Impact

- Adds PostgreSQL Operation, BookRevision, BookRevisionAuthor, AuthorRevision, BookOperationChange, and AuthorOperationChange tables, constraints, indexes, and baseline data migration.
- Changes transaction context, domain types, repository traits and PostgreSQL adapters, use-case DTOs/interactors, GraphQL schema/loaders, tests, generated schema, and architecture/database documentation.
- Changes the frontend-facing GraphQL contract; no legacy compatibility layer is retained because the primary client is controlled together with this API.
- Delivery is split into three implementation PRs: foundation and migration, Operation undo, then legacy cleanup. A final archive PR synchronizes these delta specs into the main specs.
