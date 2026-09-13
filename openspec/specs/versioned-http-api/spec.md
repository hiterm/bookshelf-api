# versioned-http-api Specification

## Purpose
Define the version namespace used by backup and future explicitly versioned
non-GraphQL HTTP APIs.

## Requirements

### Requirement: Versioned non-GraphQL HTTP APIs use a namespace
The system SHALL expose the backup APIs at `/v1/backup/snapshot` and
`/v1/backup/full`. Future non-GraphQL APIs explicitly introduced as version 1
APIs SHALL use `/v1`; existing `/me` and `/health` routes remain outside that
namespace, and `/graphql` remains unchanged. HTTP API versions SHALL be
independent of format versions carried in response bodies.

#### Scenario: Address a versioned HTTP API
- **WHEN** a client requests backup or another explicitly versioned version 1 non-GraphQL HTTP capability
- **THEN** its route begins with `/v1`
