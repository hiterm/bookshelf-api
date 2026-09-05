# revision-restore Specification

## Purpose
TBD - created by archiving change redesign-history-model. Update Purpose after archive.
## Requirements
### Requirement: Clients can restore an entity from an owned revision
The system SHALL expose Book and Author restore mutations that identify the entity and source revision number, copy the source snapshot into current state, and append a new Revision rather than making the historical Revision current.

#### Scenario: Existing Book is restored to old content
- **WHEN** an owner restores an existing Book from an earlier revision
- **THEN** the current Book matches the source content and a revision numbered one greater than the entity's current latest revision records the restored state

#### Scenario: Deleted Author is restored
- **WHEN** an owner restores a deleted Author from its last or earlier Revision
- **THEN** the Author is recreated and a new next Revision is appended

#### Scenario: Restore source is not owned
- **WHEN** a client supplies a missing or another user's Revision
- **THEN** the restore fails without changing current state or history

### Requirement: Restore preserves lifecycle semantics
The system SHALL retain the source snapshot's entity creation timestamp and SHALL set the entity update timestamp to the restore Operation time in both current state and the new Revision.

#### Scenario: Historical entity is restored
- **WHEN** a Book or Author is restored from a Revision
- **THEN** its creation time matches the source and its update time reflects the new restore

### Requirement: Book restore validates live aggregate references
The system MUST require every Author ID stored in a Book Revision to identify a currently existing Author owned by the same user and SHALL NOT restore the historical state of those Authors.

#### Scenario: Every referenced Author exists
- **WHEN** a Book Revision's Author IDs all exist for the owner
- **THEN** restore recreates exactly those current Book-Author relationships

#### Scenario: A referenced Author is deleted
- **WHEN** any Author ID in the source Book Revision does not currently exist
- **THEN** the complete restore Operation fails and rolls back

### Requirement: Restore is recorded as one atomic Operation
The system MUST record a restore Operation, the new Revision, and an OperationChange from current presence or absence to the new Revision in the same transaction as current-state writes.

#### Scenario: Restore recording fails
- **WHEN** any current-state or history write for restore fails
- **THEN** neither the restored entity nor partial restore history is committed

### Requirement: Book restore restores purchase date
The system SHALL restore a book's purchase date from the selected revision.

#### Scenario: Restore an earlier purchase date
- **WHEN** a client restores a revision containing an earlier purchase date
- **THEN** the current book has that earlier purchase date

