# operation-history Specification

## Purpose
TBD - created by archiving change redesign-history-model. Update Purpose after archive.
## Requirements
### Requirement: Logical operations are recorded independently from entity snapshots
The system SHALL record one owned Operation for each logical Book or Author mutation with a UUID, operation type, nullable typed detail, optional target Operation for undo, and database-managed creation time, and SHALL NOT store entity snapshots on the Operation.

#### Scenario: One mutation affects multiple entities
- **WHEN** an import or Author merge changes multiple Books or Authors
- **THEN** every entity change belongs to the same single Operation

#### Scenario: Operation has semantic detail
- **WHEN** an operation such as Author merge or revision restore requires operation-specific context
- **THEN** the system persists validated JSON detail corresponding to that Operation type

### Requirement: OperationChanges describe before and after revisions
The system SHALL record each affected Book and Author in an entity-specific OperationChange with nullable before and after revision numbers, and MUST require at least one side to be present.

#### Scenario: Entity is created
- **WHEN** an Operation creates an entity at revision 1
- **THEN** its change records no before revision and after revision 1

#### Scenario: Entity is updated
- **WHEN** an Operation changes an existing entity from revision N
- **THEN** its change records revision N before and revision N+1 after

#### Scenario: Entity is deleted
- **WHEN** an Operation deletes an entity at revision N
- **THEN** its change records revision N before and no after revision

### Requirement: Operation recording is transactionally atomic
The system MUST create the Operation, mutate all current entities, append all Revisions, and insert all OperationChanges in one PostgreSQL transaction owned by the use case.

#### Scenario: Any operation write fails
- **WHEN** a current-state, Revision, OperationChange, or commit write fails
- **THEN** no part of that logical Operation remains committed

### Requirement: Operations are tenant scoped and queryable
The system SHALL expose owned Operation list and single-item queries, SHALL exclude internal baseline Operations from the normal list, and SHALL resolve selected Book and Author changes through user-scoped batch loading.

#### Scenario: Scalar-only operation query
- **WHEN** a client selects only Operation scalar fields
- **THEN** the system does not query Book or Author changes

#### Scenario: Multiple operations select changes
- **WHEN** a client selects Book or Author changes for multiple Operations
- **THEN** each selected change kind is loaded in one user-scoped batch and changes expose their before and after Revisions

#### Scenario: Operation belongs to another user
- **WHEN** a client requests an Operation owned by another user
- **THEN** the system does not return that Operation or any of its changes
