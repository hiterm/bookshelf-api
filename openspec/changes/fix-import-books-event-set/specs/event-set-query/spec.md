## ADDED Requirements

### Requirement: User can query event set by ID
The system SHALL allow authenticated users to retrieve a single `event_set` record by its ID.

#### Scenario: Successful query
- **WHEN** an authenticated user sends a `eventSet(id: ID!)` query with a valid event set ID
- **THEN** the system returns the event set with `id`, `userId`, `operation`, and `createdAt` fields
- **AND** the returned event set belongs to the authenticated user

#### Scenario: Non-existent event set
- **WHEN** an authenticated user sends a `eventSet(id: ID!)` query with an ID that does not exist
- **THEN** the system returns `null`

#### Scenario: Other user's event set
- **WHEN** an authenticated user sends a `eventSet(id: ID!)` query with an ID belonging to another user
- **THEN** the system returns `null`

#### Scenario: Invalid event set ID format
- **WHEN** an authenticated user sends a `eventSet(id: ID!)` query with an invalid ID format
- **THEN** the system returns a validation error
