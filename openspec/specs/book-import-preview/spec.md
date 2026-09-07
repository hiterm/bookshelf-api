# book-import-preview Specification

## Purpose
TBD - created by archiving change add-book-import-preview. Update Purpose after archive.
## Requirements
### Requirement: Clients can preview a book import
The system SHALL expose a `previewBookImport` GraphQL mutation that accepts the
same list of `ImportBookInput` values as `importBooks` and returns preview books
without changing the `importBooks` contract.

#### Scenario: Valid preview request
- **WHEN** an authenticated client previews a valid non-empty book batch
- **THEN** the response contains one preview book for each normalized imported book with its title, authors, ISBN, read flag, owned flag, priority, format, and store

#### Scenario: Preview contract excludes transient identifiers
- **WHEN** a preview succeeds
- **THEN** its response contains no book ID, author ID, event ID, event-set ID, creation timestamp, or update timestamp

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

### Requirement: Preview reports author resolution status per book
Each preview book SHALL contain its normalized authors in input order and SHALL
label every author `EXISTING` when reused from pre-preview state or `NEW` when
created by the preview transaction.

#### Scenario: Existing and new authors are mixed
- **WHEN** a book references one existing author and one previously unknown author
- **THEN** that book reports the authors as `EXISTING` and `NEW` respectively

#### Scenario: Duplicate author within one book
- **WHEN** one book repeats the same author name
- **THEN** the preview includes that author once, matching import relationship normalization

#### Scenario: New author is shared across books
- **WHEN** multiple preview books reference the same previously unknown author
- **THEN** every one of those books reports that author as `NEW`

### Requirement: Preview results are advisory
The system SHALL evaluate every preview and import in its own transaction
against the database state visible to that execution and SHALL NOT reserve or
lock a preview result for a later import.

#### Scenario: State changes between preview and import
- **WHEN** an author reported as `NEW` by preview is created before the client runs import
- **THEN** the later import reuses that now-existing author and succeeds according to its own transaction state

### Requirement: Preview execution has no transaction-external side effects
The shared import execution path used by preview MUST NOT perform external API
calls, message publication, email, file writes, out-of-transaction database
writes, external event publication, or irreversible audit writes.

#### Scenario: Preview is rolled back
- **WHEN** preview execution completes and its database transaction is rolled back
- **THEN** no side effect from that execution survives outside the transaction

### Requirement: Preview returns purchase dates
The system SHALL return each requested purchase date in book import previews.

#### Scenario: Preview a purchase date
- **WHEN** a client previews a book with a purchase date
- **THEN** the preview returns that date without persisting the book

