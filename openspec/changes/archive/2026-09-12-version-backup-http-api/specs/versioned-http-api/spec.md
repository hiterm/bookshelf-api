## ADDED Requirements

### Requirement: Non-GraphQL HTTP APIs use a version namespace
The system SHALL expose version 1 non-GraphQL HTTP APIs below `/v1` while keeping `/graphql` unchanged. HTTP API versions SHALL be independent of format versions carried in response bodies.

#### Scenario: Address a versioned HTTP API
- **WHEN** a client requests a version 1 non-GraphQL HTTP capability
- **THEN** its route begins with `/v1`
