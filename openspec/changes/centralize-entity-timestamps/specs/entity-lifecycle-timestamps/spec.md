## ADDED Requirements

### Requirement: Use cases own entity lifecycle time
The system SHALL have each Book and Author mutation use case choose one UTC lifecycle timestamp for its logical operation, and domain entities and repositories MUST NOT independently replace that timestamp.

#### Scenario: Create one entity
- **WHEN** a Book or Author is created
- **THEN** its creation and update timestamps are the same operation timestamp

#### Scenario: Update an entity
- **WHEN** a Book or Author is updated
- **THEN** its creation timestamp remains unchanged and its update timestamp becomes the operation timestamp

#### Scenario: Import multiple entities
- **WHEN** one import operation creates multiple Books or Authors
- **THEN** every entity created by that operation uses the same creation and update timestamp

#### Scenario: Import references an existing author
- **WHEN** an import resolves an Author that already exists
- **THEN** the existing Author's creation and update timestamps remain unchanged

### Requirement: Restore records a new lifecycle update
The system SHALL preserve an entity's historical creation timestamp when restoring a Book or Author snapshot and SHALL set its update timestamp to the restore operation timestamp.

#### Scenario: Restore an entity snapshot
- **WHEN** a Book or Author is restored from a non-delete history event
- **THEN** the restored entity retains the snapshot creation timestamp and receives the restore operation timestamp as its update timestamp

#### Scenario: Restore a deleted state
- **WHEN** a delete history event is restored and the entity becomes absent
- **THEN** the system does not create entity lifecycle timestamps for that absent entity

### Requirement: Audit time remains database-managed
The system SHALL treat event-set creation timestamps and entity-event change timestamps as database-recording times independent of entity lifecycle timestamps.

#### Scenario: Record a mutation event
- **WHEN** a Book or Author mutation is committed
- **THEN** PostgreSQL records the event audit timestamp without requiring it to equal the entity lifecycle timestamp
