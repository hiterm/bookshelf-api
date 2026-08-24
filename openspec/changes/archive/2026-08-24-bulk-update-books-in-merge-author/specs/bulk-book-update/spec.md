## ADDED Requirements

### Requirement: Bulk Book updates preserve aggregate state
The system SHALL persist every supplied updated Book's scalar fields and final
author relationships in one transaction, scoped to the transaction's user,
without changing Book creation timestamps.

#### Scenario: Multiple Books have distinct final states
- **WHEN** multiple updated Books with different scalar fields and author sets are persisted together
- **THEN** each Book and its author relationships match that Book's supplied final state

#### Scenario: A Book has no final authors
- **WHEN** a targeted Book is bulk-updated with an empty author set
- **THEN** all existing author relationships for that Book are removed

#### Scenario: A relationship already exists
- **WHEN** a supplied final author relationship already exists in `book_author`
- **THEN** the update succeeds and exactly one relationship remains under the existing primary key constraint

#### Scenario: Input is empty
- **WHEN** no Books are supplied to the bulk update
- **THEN** the operation succeeds without changing persistent state or recording events

#### Scenario: A Book is not owned by the transaction user
- **WHEN** the bulk update includes a Book not owned by the transaction user
- **THEN** the operation fails without changing that Book or creating dependent relationship or event records for it

### Requirement: Bulk Book updates record equivalent events
The system SHALL record one `update` Book event for every successfully updated
Book using the transaction's event set and the Book's supplied post-update
snapshot, including its final author identifiers.

#### Scenario: Multiple Book update events are recorded
- **WHEN** multiple Books are successfully persisted together
- **THEN** each Book has one update event in the same event set with scalar and author snapshots matching its final state

#### Scenario: A Book event has no authors
- **WHEN** a successfully updated Book has an empty final author set
- **THEN** its update event is recorded without `book_event_author` rows

### Requirement: Bulk persistence uses set-based database operations
The system SHALL persist Book rows, relationships, Book events, and event-author
snapshots without executing a database statement once per supplied Book.

#### Scenario: The number of Books increases
- **WHEN** the bulk update receives more Books
- **THEN** SQL round trips for the update remain bounded by the fixed bulk persistence stages rather than increasing per Book

### Requirement: Author merge uses bulk Book persistence
The system SHALL continue to apply `Book::update()` to each Book linked to the
source Author and SHALL persist the resulting Book collection through one
general bulk repository call while retaining the existing Book-before-Author
lock order.

#### Scenario: Source Books are merged into a destination Author
- **WHEN** an Author with multiple linked Books is merged into a different Author
- **THEN** every Book removes the source Author, contains the destination Author exactly once, and is included in one bulk persistence call

#### Scenario: The source Author has no Books
- **WHEN** an Author with no linked Books is merged
- **THEN** one empty bulk persistence call succeeds and the existing Author merge behavior continues
