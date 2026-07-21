## ADDED Requirements

### Requirement: GraphQL authors expose lifecycle timestamps
The system SHALL expose each author's persisted creation and update timestamps through the `createdAt` and `updatedAt` fields of the GraphQL `Author` type as Unix seconds.

#### Scenario: Query an author
- **WHEN** an authenticated client queries an existing author and selects `createdAt` and `updatedAt`
- **THEN** the system returns the timestamps persisted for that author

#### Scenario: Create an author
- **WHEN** an authenticated client creates an author and selects `author.createdAt` and `author.updatedAt` from the mutation payload
- **THEN** the system returns non-null timestamps matching the newly persisted author

#### Scenario: Update an author
- **WHEN** an authenticated client updates an author and selects `author.createdAt` and `author.updatedAt` from the mutation payload
- **THEN** the creation timestamp remains unchanged and the update timestamp reflects the persisted update
