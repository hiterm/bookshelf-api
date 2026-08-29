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
The system MUST deduplicate requested author names, use the tenant-scoped author uniqueness constraint to insert missing authors, resolve all requested author IDs and created-versus-existing statuses in one lookup, and record an Author Revision and OperationChange only for each Author created by the import or preview execution.

#### Scenario: Existing and new authors are mixed
- **WHEN** an import or preview references both existing and previously unknown author names
- **THEN** all names resolve to their tenant-scoped IDs and statuses and only newly inserted Authors receive Author Revisions and changes

#### Scenario: Author names repeat
- **WHEN** one author name occurs in multiple books or more than once in one book
- **THEN** the name is resolved once and each book contains that author relationship at most once

#### Scenario: Imports race to create an author
- **WHEN** concurrent import executions request the same previously unknown tenant-scoped author name
- **THEN** the database uniqueness constraint prevents duplicates and each execution resolves the surviving author ID with the status produced by its own find-or-create operation

### Requirement: Bulk book persistence records complete snapshots
The system SHALL bulk-insert all Books, Book-Author relationships, one revision 1 per Book, matching revision-author relationships, and one BookOperationChange per Book while preserving the same complete snapshots as single-Book creation.

#### Scenario: Books have varied author counts
- **WHEN** an import contains Books with zero, one, and multiple Authors
- **THEN** every Book and all applicable current and Revision Author relationships are persisted correctly

#### Scenario: Changes share the import Operation
- **WHEN** Books and new Authors are persisted by one import
- **THEN** every entity OperationChange references the single `ImportBooks` Operation

### Requirement: Bulk import remains atomic and compatible
The system MUST preserve the `importBooks` validation, response entity semantics, 1,000-Book limit, and single-transaction behavior while replacing Event identifiers with Operation metadata and sharing its validation and transactional execution path with book import preview.

#### Scenario: A bulk statement fails
- **WHEN** any Author, Book, relationship, Operation, Revision, or OperationChange statement fails before commit
- **THEN** the complete import is rolled back without partial rows

#### Scenario: Validation fails
- **WHEN** an import request violates an existing validation rule
- **THEN** the system returns the existing validation error without beginning a transaction

#### Scenario: Import succeeds after preview
- **WHEN** a valid batch is previewed and then imported without intervening database changes
- **THEN** `importBooks` commits the same normalized Books and Author relationships and returns its Books and Operation ID
