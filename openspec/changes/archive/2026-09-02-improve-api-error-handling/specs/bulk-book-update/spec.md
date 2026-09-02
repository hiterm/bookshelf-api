## MODIFIED Requirements

### Requirement: Bulk Book updates preserve aggregate state
The system SHALL require unique input Book IDs, persist every supplied updated Book's scalar fields and final Author relationships in one transaction scoped to the transaction's user without changing Book creation timestamps, and report an actually absent or out-of-scope input Book ID when any target cannot be updated.

#### Scenario: Multiple Books have distinct final states
- **WHEN** multiple updated Books with different scalar fields and Author sets are persisted together
- **THEN** each Book and its Author relationships match that Book's supplied final state

#### Scenario: A Book has no final authors
- **WHEN** a targeted Book is bulk-updated with an empty Author set
- **THEN** all existing Author relationships for that Book are removed

#### Scenario: A relationship already exists
- **WHEN** a supplied final Author relationship already exists in `book_author`
- **THEN** the update succeeds and exactly one relationship remains under the existing primary key constraint

#### Scenario: Input is empty
- **WHEN** no Books are supplied to the bulk update
- **THEN** the operation succeeds without changing persistent state or recording Revisions or OperationChanges

#### Scenario: An input Book ID is duplicated
- **WHEN** the same Book ID appears more than once in a bulk update
- **THEN** the operation fails with Validation before changing persistent state or recording Revisions or OperationChanges

#### Scenario: A later Book does not exist
- **WHEN** the first supplied Book exists for the transaction user and a later supplied Book does not exist
- **THEN** the operation fails with NotFound identifying the actually missing later Book ID and rolls back all updates

#### Scenario: A Book is not owned by the transaction user
- **WHEN** the bulk update includes a Book not owned by the transaction user
- **THEN** the operation fails with NotFound identifying that out-of-scope Book ID without changing it or creating dependent relationship, Revision, or OperationChange records
