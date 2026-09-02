# bulk-book-update Specification

## Purpose
TBD - created by archiving change bulk-update-books-in-merge-author. Update Purpose after archive.
## Requirements
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

### Requirement: Bulk Book updates record equivalent revisions
The system SHALL record one complete Book Revision and one BookOperationChange for every successfully updated Book using the transaction's Operation and the Book's supplied post-update snapshot, including final Author identifiers.

#### Scenario: Multiple Book revisions are recorded
- **WHEN** multiple Books are successfully persisted together
- **THEN** each Book has one new Revision and change in the same Operation with scalar and Author snapshots matching its final state

#### Scenario: A Book revision has no authors
- **WHEN** a successfully updated Book has an empty final Author set
- **THEN** its Revision is recorded without `book_revision_author` rows

### Requirement: Bulk persistence uses set-based database operations
The system SHALL persist Book rows, relationships, Book Revisions, revision-author snapshots, and OperationChanges without executing a database statement once per supplied Book.

#### Scenario: The number of Books increases
- **WHEN** the bulk update receives more Books
- **THEN** SQL round trips for the update remain bounded by fixed bulk persistence stages rather than increasing per Book

### Requirement: Author merge uses bulk Book persistence
The system SHALL continue to apply `Book::update()` to each Book linked to the source Author and SHALL persist the resulting Book collection through one general bulk repository call while retaining deterministic Book-before-Author lock order.

#### Scenario: Source Books are merged into a destination Author
- **WHEN** an Author with multiple linked Books is merged into a different Author
- **THEN** every Book removes the source Author, contains the destination Author exactly once, and receives a Revision and OperationChange in the merge Operation through one bulk persistence call

#### Scenario: The source Author has no Books
- **WHEN** an Author with no linked Books is merged
- **THEN** one empty bulk persistence call succeeds and the Author merge records its remaining entity changes
