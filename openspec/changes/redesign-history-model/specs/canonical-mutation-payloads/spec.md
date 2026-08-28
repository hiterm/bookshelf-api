## MODIFIED Requirements

### Requirement: Book mutation payloads expose one entity representation
The system SHALL expose the affected Book from create and update mutations through the `book` field of `BookMutationPayload`, SHALL expose the logical Operation through `operationId`, SHALL expose the newly recorded Book revision through `revisionNumber`, and SHALL NOT expose Event identifiers or direct aliases of fields belonging to that Book.

#### Scenario: Create Book returns canonical payload
- **WHEN** an authenticated client successfully executes `createBook`
- **THEN** the client can select the created identifier through `createBook.book.id`, the Operation through `createBook.operationId`, and revision 1 through `createBook.revisionNumber`

#### Scenario: Update Book returns canonical payload
- **WHEN** an authenticated client successfully executes `updateBook`
- **THEN** the client can select updated entity fields through `updateBook.book` and its new revision metadata without direct entity-field or Event-ID aliases

### Requirement: Author mutation payloads expose one entity representation
The system SHALL expose the affected Author from create and update mutations through the `author` field of `AuthorMutationPayload`, SHALL expose `operationId` and `revisionNumber`, and SHALL NOT expose Event identifiers or direct aliases of fields belonging to that Author.

#### Scenario: Create Author returns canonical payload
- **WHEN** an authenticated client successfully executes `createAuthor`
- **THEN** the client can select the created Author, Operation ID, and revision 1 from canonical fields

#### Scenario: Update Author returns canonical payload
- **WHEN** an authenticated client successfully executes `updateAuthor`
- **THEN** the client can select the updated Author and new revision metadata without direct entity-field aliases

### Requirement: Delete payloads use descriptive entity identifiers
The system SHALL expose the deleted entity through `bookId` or `authorId`, SHALL expose `operationId`, and SHALL NOT define a generic `id`, `eventId`, or `eventSetId` alias.

#### Scenario: Delete Book returns its identifier
- **WHEN** an authenticated client successfully executes `deleteBook`
- **THEN** the client can select `bookId` and `operationId`

#### Scenario: Delete Author returns its identifier
- **WHEN** an authenticated client successfully executes `deleteAuthor`
- **THEN** the client can select `authorId` and `operationId`

### Requirement: New mutation contract is authoritative from PR 1
All mutations SHALL expose only Operation and Revision metadata from PR 1. Legacy Event/EventSet tables and internal code MAY remain until PR 3, but their GraphQL APIs SHALL be absent from PR 1. Mutations SHALL NOT expose `eventId` or `eventSetId` aliases and SHALL NOT write legacy history.

#### Scenario: Existing frontend selects removed metadata
- **WHEN** an older frontend selects `eventSetId` from any mutation after PR 1
- **THEN** GraphQL validation fails because no compatibility alias exists

#### Scenario: Mutation records history while legacy tables remain
- **WHEN** a client creates, updates, deletes, restores, imports, or merges entities before PR 3 cleanup
- **THEN** only Operation, Revision, and OperationChange rows are written
