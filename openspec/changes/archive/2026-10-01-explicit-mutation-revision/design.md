## Context

Repositories already allocate independent revisions per owned entity. Single-entity create/update return raw integers, restore returns only the entity, and use cases read a mutable revision slot on the transaction. The unit transaction always reports revision 1.

## Goals / Non-Goals

**Goals:** Make revision provenance explicit, preserve atomic writes and public payloads, and verify non-first revisions.

**Non-Goals:** Change undo boundaries, database allocation/locking, bulk SQL, restore input types, or GraphQL schema.

## Decisions

- Create/update return the existing validated `RevisionNumber`. History append helpers construct this type from the allocated database number and return it after successful writes. Keeping raw integers would leave this internal contract less precise.
- A shared domain repository `RevisionMutationResult<T>` carries `entity` and `revision_number` for restore. A named result makes provenance clearer than a tuple and serves both entities.
- Transactions carry only operation/user context. Remove the mutable revision field, getter, setter, and constant mock result. Bulk operations keep their aggregate contracts without a last-revision concept.
- Use cases convert explicit revision numbers to integers only when constructing response DTOs, and return success only after commit.
- Unit tests use distinct create/update/restore values; database tests compare returned revisions to immutable snapshots and OperationChanges, including multiple entities in one transaction.

## Risks / Trade-offs

- [Cross-layer signature changes affect mocks and database tests] → Compile all targets and database-feature tests; preserve existing error/commit tests.
- [Restore accidentally returns the requested source number] → Assert the new result differs from the source and matches persisted history.

## Migration Plan

No database or public API migration. Deploy as an ordinary code change; rollback restores the previous binary.

## Open Questions

E2E augmentation was offered to the user under AGENTS.md. In the absence of a response, retain and run existing E2E coverage: the restore cases already assert revision 3, and the added unit/database assertions target the changed internal contract.
