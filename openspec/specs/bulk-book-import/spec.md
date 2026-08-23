# bulk-book-import Specification

## Purpose
Define bounded-query persistence for `importBooks` while preserving its API,
transaction, relationship, and event-recording semantics.

## Requirements
### Requirement: Import persistence uses bounded database statements
The system SHALL persist an `importBooks` request with a number of database
statements that does not grow with either the number of books or the number of
unique author names.

#### Scenario: Import many books and authors
- **WHEN** a valid import contains multiple books and multiple unique authors
- **THEN** the system bulk-resolves authors once and bulk-creates books once without awaited database access inside an input-sized loop

#### Scenario: Import the maximum batch
- **WHEN** a valid import contains 1,000 books
- **THEN** the system persists the complete batch through the bounded bulk path

### Requirement: Bulk author resolution preserves creation semantics
The system MUST deduplicate requested author names, use the tenant-scoped
author uniqueness constraint to insert missing authors, resolve all requested
author IDs in one lookup, and record an author event only for each author
created by the import.

#### Scenario: Existing and new authors are mixed
- **WHEN** an import references both existing and previously unknown author names
- **THEN** all names resolve to their tenant-scoped IDs and only newly inserted authors receive author events

#### Scenario: Author names repeat
- **WHEN** one author name occurs in multiple books or more than once in one book
- **THEN** the name is resolved once and each book contains that author relationship at most once

#### Scenario: Imports race to create an author
- **WHEN** concurrent imports request the same previously unknown tenant-scoped author name
- **THEN** the database uniqueness constraint prevents duplicates and each import resolves the surviving author ID

### Requirement: Bulk book persistence records complete snapshots
The system SHALL bulk-insert all books, book-author relationships, one create
event per book, and matching event-author relationships while preserving the
same snapshots as single-book creation.

#### Scenario: Books have varied author counts
- **WHEN** an import contains books with zero, one, and multiple authors
- **THEN** every book and all applicable current and event-snapshot author relationships are persisted correctly

#### Scenario: Events share the import event set
- **WHEN** books and new authors are persisted by one import
- **THEN** every entity event references the single event set created for `ImportBooks`

### Requirement: Bulk import remains atomic and compatible
The system MUST preserve the existing GraphQL schema, validation, response
semantics, 1,000-book limit, and single-transaction behavior.

#### Scenario: A bulk statement fails
- **WHEN** any author, book, relationship, or event statement fails before commit
- **THEN** the complete import is rolled back without partial rows

#### Scenario: Validation fails
- **WHEN** an import request violates an existing validation rule
- **THEN** the system returns the existing validation error without beginning a transaction
