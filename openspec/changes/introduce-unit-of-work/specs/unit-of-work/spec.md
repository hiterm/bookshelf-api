## ADDED Requirements

### Requirement: Transaction lifecycle management
The system SHALL provide a mechanism to begin, commit, and roll back a database transaction as a single unit of work.

#### Scenario: Successful commit
- **WHEN** a unit of work is started and all operations within it succeed
- **THEN** the system commits the transaction and persists all changes atomically

#### Scenario: Rollback on failure
- **WHEN** a unit of work is started and any operation within it fails
- **THEN** the system rolls back the transaction and no partial changes are persisted

### Requirement: Cross-repository transaction sharing
The system SHALL allow multiple aggregate repositories to execute operations within the same shared transaction.

#### Scenario: Book and author creation in shared transaction
- **WHEN** a book repository and an author repository perform write operations within the same unit of work
- **THEN** both operations use the same underlying database transaction
- **AND** the transaction is committed or rolled back as a single atomic unit

### Requirement: Event recording within shared transaction
The system SHALL record entity change events within the same transaction as the entity mutation.

#### Scenario: Book creation with event in shared transaction
- **WHEN** a book is created within a unit of work
- **THEN** the book row and its corresponding `book_event` row are inserted within the same transaction
- **AND** if the transaction is rolled back, neither the book nor the event remains in the database

### Requirement: Backward compatibility for existing repository interfaces
The system SHALL maintain existing domain repository traits without modification.

#### Scenario: Pool-based repository operations continue to work
- **WHEN** existing code calls a repository method that internally manages its own transaction
- **THEN** the operation completes successfully without requiring any changes to the caller
- **AND** the repository begins and commits its own transaction as before

### Requirement: Import books via shared transaction
The system SHALL support importing books and upserting authors within a single transaction.

#### Scenario: Import deduplicates authors and creates books atomically
- **WHEN** multiple books with overlapping author names are imported within a unit of work
- **THEN** each author is created at most once
- **AND** all books are created
- **AND** corresponding events are recorded for both authors and books
- **AND** the entire operation is atomic

#### Scenario: Import rolls back on duplicate book ID
- **WHEN** an import operation contains duplicate book IDs within a unit of work
- **THEN** the operation fails
- **AND** no authors, books, or events are persisted
