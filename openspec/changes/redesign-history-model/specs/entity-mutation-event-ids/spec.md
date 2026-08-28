## REMOVED Requirements

### Requirement: Create mutations expose their recorded entity event
**Reason**: Entity Events are replaced by per-entity Revisions grouped through Operations.

**Migration**: Clients use `operationId` and `revisionNumber` from create mutation payloads.

### Requirement: Update mutations expose their recorded entity event
**Reason**: Entity Events are replaced by per-entity Revisions grouped through Operations.

**Migration**: Clients use `operationId` and `revisionNumber` from update mutation payloads.

### Requirement: Returned entity event IDs are valid restore sources
**Reason**: Restore sources are now identified by entity ID and revision number, not Event ID.

**Migration**: Clients pass `bookId` or `authorId` together with `revisionNumber` to the corresponding restore mutation.

### Requirement: Entity event IDs remain transactionally atomic
**Reason**: No mutation returns an entity Event ID after the history redesign.

**Migration**: Atomic Operation, Revision, and OperationChange recording is specified by `operation-history` and `entity-revision-history`.

### Requirement: Multi-event mutation contracts remain unchanged
**Reason**: The GraphQL contract intentionally makes a breaking migration from Event/EventSet identifiers to Operation/Revision metadata.

**Migration**: Multi-entity mutations return their single logical `operationId`; they do not claim one entity revision number.

