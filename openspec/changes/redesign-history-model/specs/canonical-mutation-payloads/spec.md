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

### Requirement: Known client migrates before Event contract removal
The `bookshelf` client SHALL use Operation and Revision mutation metadata before `bookshelf-api` removes Event/EventSet fields. Until that client migration lands, bulk import and author merge payloads SHALL retain `eventSetId` as a deprecated alias of `operationId`; no new client SHALL depend on the alias, and it SHALL be removed with the legacy Event contract.

#### Scenario: Frontend mutates an entity
- **WHEN** the frontend creates, updates, deletes, restores, imports, merges, or undoes entities
- **THEN** its GraphQL documents and test doubles use the canonical entity, `operationId`, and applicable `revisionNumber` fields

#### Scenario: Existing client uses a bulk-operation payload during migration
- **WHEN** the deployed frontend selects `eventSetId` from `importBooks` or `mergeAuthor` before its migration lands
- **THEN** the API returns the same identifier as `operationId` through the deprecated alias without creating a second Event or Operation

### Requirement: Test doubles match the canonical schema
Frontend test doubles SHALL return the same Operation/Revision payload nesting and identifier names as the production GraphQL schema.

#### Scenario: Mocked mutation responses
- **WHEN** unit, demo-mode, or mock-API tests execute a Book or Author mutation
- **THEN** responses expose canonical entity and Operation/Revision metadata without Event identifiers
