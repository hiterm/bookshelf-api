## MODIFIED Requirements

### Requirement: Bulk author resolution preserves creation semantics
The system MUST deduplicate requested author names, use the tenant-scoped
author uniqueness constraint to insert missing authors, resolve all requested
author IDs and created-versus-existing statuses in one lookup, and record an
author event only for each author created by the import or preview execution.

#### Scenario: Existing and new authors are mixed
- **WHEN** an import or preview references both existing and previously unknown author names
- **THEN** all names resolve to their tenant-scoped IDs and statuses and only newly inserted authors receive author events

#### Scenario: Author names repeat
- **WHEN** one author name occurs in multiple books or more than once in one book
- **THEN** the name is resolved once and each book contains that author relationship at most once

#### Scenario: Imports race to create an author
- **WHEN** concurrent import executions request the same previously unknown tenant-scoped author name
- **THEN** the database uniqueness constraint prevents duplicates and each execution resolves the surviving author ID with the status produced by its own find-or-create operation

### Requirement: Bulk import remains atomic and compatible
The system MUST preserve the existing `importBooks` GraphQL schema, validation,
response semantics, 1,000-book limit, and single-transaction behavior while
sharing its validation and transactional execution path with book import
preview.

#### Scenario: A bulk statement fails
- **WHEN** any author, book, relationship, or event statement fails before commit
- **THEN** the complete import is rolled back without partial rows

#### Scenario: Validation fails
- **WHEN** an import request violates an existing validation rule
- **THEN** the system returns the existing validation error without beginning a transaction

#### Scenario: Import succeeds after preview
- **WHEN** a valid batch is previewed and then imported without intervening database changes
- **THEN** `importBooks` commits the same normalized books and author relationships and returns its existing books and event-set payload
