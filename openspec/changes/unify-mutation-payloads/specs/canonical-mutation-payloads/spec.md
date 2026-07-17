## ADDED Requirements

### Requirement: Book mutation payloads expose one entity representation
The system SHALL expose the affected book from create and update mutations through the `book` field of `BookMutationPayload`, SHALL expose the associated event set through `eventSetId`, and SHALL NOT expose direct aliases of fields belonging to that book on the payload.

#### Scenario: Create book returns canonical payload
- **WHEN** an authenticated client successfully executes `createBook`
- **THEN** the client can select the created identifier through `createBook.book.id`
- **AND** the client can select the mutation event set through `createBook.eventSetId`

#### Scenario: Update book returns canonical payload
- **WHEN** an authenticated client successfully executes `updateBook`
- **THEN** the client can select updated entity fields through `updateBook.book`
- **AND** the payload does not define direct fields such as `id`, `title`, or `updatedAt`

### Requirement: Author mutation payloads expose one entity representation
The system SHALL expose the affected author from create and update mutations through the `author` field of `AuthorMutationPayload`, SHALL expose the associated event set through `eventSetId`, and SHALL NOT expose direct aliases of fields belonging to that author on the payload.

#### Scenario: Create author returns canonical payload
- **WHEN** an authenticated client successfully executes `createAuthor`
- **THEN** the client can select the created identifier through `createAuthor.author.id`
- **AND** the client can select the mutation event set through `createAuthor.eventSetId`

#### Scenario: Update author returns canonical payload
- **WHEN** an authenticated client successfully executes `updateAuthor`
- **THEN** the client can select updated entity fields through `updateAuthor.author`
- **AND** the payload does not define direct `id` or `name` fields

### Requirement: Delete payloads use descriptive entity identifiers
The system SHALL expose a deleted book identifier as `bookId`, SHALL expose a deleted author identifier as `authorId`, SHALL preserve `eventSetId` on both payloads, and SHALL NOT define a generic `id` alias on either delete payload.

#### Scenario: Delete book returns its identifier
- **WHEN** an authenticated client successfully executes `deleteBook`
- **THEN** the client can select the deleted identifier through `deleteBook.bookId`
- **AND** the client can select the mutation event set through `deleteBook.eventSetId`

#### Scenario: Delete author returns its identifier
- **WHEN** an authenticated client successfully executes `deleteAuthor`
- **THEN** the client can select the deleted identifier through `deleteAuthor.authorId`
- **AND** the client can select the mutation event set through `deleteAuthor.eventSetId`

### Requirement: Known client migrates before alias removal
The `bookshelf` client SHALL use only canonical mutation payload fields before `bookshelf-api` removes the compatibility aliases.

#### Scenario: Frontend creates a book and pending authors
- **WHEN** a user creates a book and the form creates one or more new authors
- **THEN** the frontend reads author identifiers through `createAuthor.author.id`
- **AND** it reads the created book identifier through `createBook.book.id`

#### Scenario: Frontend updates and deletes entities
- **WHEN** the frontend executes book or author update and delete mutations
- **THEN** its GraphQL documents select updated entities through `book` or `author`
- **AND** its delete documents select `bookId` or `authorId`

### Requirement: Test doubles match the canonical schema
Frontend test doubles SHALL return the same mutation payload nesting and identifier names as the production GraphQL schema.

#### Scenario: Mocked mutation responses
- **WHEN** unit, demo-mode, or mock-API tests execute a book or author mutation
- **THEN** create and update responses wrap entities under `book` or `author`
- **AND** delete responses expose `bookId` or `authorId`
