## MODIFIED Requirements

### Requirement: GraphQL errors provide stable machine-readable codes
The system SHALL add a stable `extensions.code` to every GraphQL error produced from a PresentationalError through query, mutation, or nested loader-backed resolver execution.

#### Scenario: A requested entity is absent
- **WHEN** a NotFound error reaches the GraphQL boundary through a resolver
- **THEN** its extension code is `NOT_FOUND` and its path identifies the invoked resolver field

#### Scenario: Client input is invalid
- **WHEN** a Validation error reaches the GraphQL boundary through a resolver
- **THEN** its extension code is `VALIDATION_ERROR` and its path identifies the invoked resolver field

#### Scenario: An operation conflicts with current state
- **WHEN** a Conflict error reaches the GraphQL boundary through a resolver
- **THEN** its extension code is `CONFLICT` and its path identifies the invoked resolver field

#### Scenario: An internal failure reaches GraphQL
- **WHEN** an InfrastructureError or Unexpected error reaches the GraphQL boundary through a resolver
- **THEN** its extension code is `INTERNAL_ERROR` and its path identifies the invoked resolver field

#### Scenario: A nested loader fails
- **WHEN** a PresentationalError is returned while resolving a loader-backed nested field
- **THEN** the error uses the same public extension code and includes the complete nested resolver path

### Requirement: GraphQL errors expose only safe public information
The system SHALL preserve actionable validation, conflict, entity type, and entity ID information while excluding tenant identifiers and internal failure details from GraphQL error messages returned by query, mutation, and loader-backed resolver execution.

#### Scenario: NotFound contains internal tenant context
- **WHEN** a NotFound error containing a user ID reaches the GraphQL boundary through a resolver
- **THEN** the public message identifies the missing entity without containing the user ID

#### Scenario: Infrastructure failure contains database details
- **WHEN** an InfrastructureError containing database, SQL, or internal-state details reaches the GraphQL boundary through a resolver or loader
- **THEN** the public message is `Internal server error` and contains none of those details

#### Scenario: Unexpected failure contains an internal message
- **WHEN** an Unexpected error reaches the GraphQL boundary through a resolver
- **THEN** the public message is `Internal server error` and does not contain the internal message

## ADDED Requirements

### Requirement: Business-error tests prove resolver execution
HTTP tests for GraphQL business errors MUST use a valid operation document and selection set, require a non-empty errors array, assert the expected `extensions.code` and resolver `path`, and demonstrate with a valid control input that the target operation can succeed.

#### Scenario: Import contains one invalid entry
- **WHEN** an import mutation with a valid selection set contains one invalid book among valid books
- **THEN** the response reports `VALIDATION_ERROR` at the `importBooks` path and persists no imported books, authors, or operation

#### Scenario: A representative validation fails over HTTP
- **WHEN** a syntactically and structurally valid create mutation contains invalid business input
- **THEN** the response reports `VALIDATION_ERROR` at the mutation field path rather than a parsing or schema-validation error

#### Scenario: A representative conflict fails over HTTP
- **WHEN** a syntactically and structurally valid mutation conflicts with existing state
- **THEN** the response reports `CONFLICT` at the mutation field path rather than a parsing or schema-validation error

#### Scenario: Valid control input succeeds
- **WHEN** the corresponding operation is sent with valid non-conflicting input
- **THEN** it completes without GraphQL errors

