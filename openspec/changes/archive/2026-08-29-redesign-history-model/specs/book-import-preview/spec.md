## MODIFIED Requirements

### Requirement: Preview executes the real import path and rolls it back
The system MUST run preview and import through the same validation, bulk author resolution, book construction, repository persistence, database constraints, Operation creation, Revision recording, and OperationChange recording path, and MUST explicitly roll back a successfully executed preview transaction.

#### Scenario: Preview reaches transactional persistence
- **WHEN** valid input is previewed
- **THEN** author, book, relationship, Operation, Revision, revision-author, and OperationChange writes execute inside one transaction using `ImportBooks` semantics before rollback

#### Scenario: Database state is unchanged
- **WHEN** a preview completes successfully
- **THEN** no Author, Book, Book-Author relationship, Operation, Revision, revision-author relationship, or OperationChange created by the preview remains in the database

#### Scenario: Validation parity
- **WHEN** preview and import receive the same empty, oversized, or invalid batch
- **THEN** both reject it with the same validation semantics without beginning a transaction

#### Scenario: Constraint parity
- **WHEN** the shared persistence path violates a database constraint
- **THEN** preview fails as import would and leaves no partial state

#### Scenario: Rollback fails
- **WHEN** successful preview execution is followed by a rollback failure
- **THEN** the system returns the rollback database error and does not report a successful preview

