# api-error-contract Specification

## Purpose
TBD - created by archiving change improve-api-error-handling. Update Purpose after archive.
## Requirements
### Requirement: Expected database conflicts are classified in operation context
The system SHALL translate a database constraint violation into a domain error only when the repository operation and exact named constraint establish a client-actionable meaning, and SHALL retain infrastructure classification for unexpected database errors.

#### Scenario: Author name conflicts during creation
- **WHEN** an author creation violates `author_user_id_name_unique`
- **THEN** the operation fails with a Conflict error describing that the author name is already in use

#### Scenario: Author name conflicts during update
- **WHEN** an author update violates `author_user_id_name_unique`
- **THEN** the operation fails with a Conflict error describing that the author name is already in use

#### Scenario: An unknown database failure occurs
- **WHEN** a repository receives a database error that is not an explicitly recognized constraint in that operation
- **THEN** the error remains an InfrastructureError and is not inferred to be a client conflict from SQLSTATE alone

### Requirement: GraphQL errors provide stable machine-readable codes
The system SHALL add a stable `extensions.code` to every GraphQL error produced from a PresentationalError through the shared presentation conversion.

#### Scenario: A requested entity is absent
- **WHEN** a NotFound error reaches the GraphQL boundary
- **THEN** its extension code is `NOT_FOUND`

#### Scenario: Client input is invalid
- **WHEN** a Validation error reaches the GraphQL boundary
- **THEN** its extension code is `VALIDATION_ERROR`

#### Scenario: An operation conflicts with current state
- **WHEN** a Conflict error reaches the GraphQL boundary
- **THEN** its extension code is `CONFLICT`

#### Scenario: An internal failure reaches GraphQL
- **WHEN** an InfrastructureError or Unexpected error reaches the GraphQL boundary
- **THEN** its extension code is `INTERNAL_ERROR`

### Requirement: GraphQL errors expose only safe public information
The system SHALL preserve actionable validation, conflict, entity type, and entity ID information while excluding tenant identifiers and internal failure details from GraphQL error messages.

#### Scenario: NotFound contains internal tenant context
- **WHEN** a NotFound error containing a user ID reaches the GraphQL boundary
- **THEN** the public message identifies the missing entity without containing the user ID

#### Scenario: Infrastructure failure contains database details
- **WHEN** an InfrastructureError containing database, SQL, or internal-state details reaches the GraphQL boundary
- **THEN** the public message is `Internal server error` and contains none of those details

#### Scenario: Unexpected failure contains an internal message
- **WHEN** an Unexpected error reaches the GraphQL boundary
- **THEN** the public message is `Internal server error` and does not contain the internal message

### Requirement: Internal error causes remain diagnosable
The system SHALL retain internal error causes and record internal GraphQL failures through server tracing without logging the same error in multiple conversion layers.

#### Scenario: An internal error is sanitized
- **WHEN** an InfrastructureError or Unexpected error is converted to its safe GraphQL representation
- **THEN** the original diagnostic detail is emitted to server tracing while only the fixed public message is returned

