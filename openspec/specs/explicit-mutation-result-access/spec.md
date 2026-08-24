# explicit-mutation-result-access Specification

## Purpose
Define explicit access boundaries between mutation result containers and their
inner values while preserving mutation API and event-recording behavior.

## Requirements
### Requirement: Mutation result values use explicit access
Mutation result DTOs SHALL expose their inner mutation value through the
`value` field without implementing transparent dereferencing to that value.

#### Scenario: Access an inner DTO field
- **WHEN** application code reads a field belonging to the inner mutation DTO
- **THEN** it accesses that field through the mutation result's `value` field

### Requirement: Event metadata remains on the result container
Mutation result DTOs SHALL continue to expose `event_set_id` and, where
applicable, `event_id` as fields of the result container.

#### Scenario: Access mutation event metadata
- **WHEN** application code constructs a mutation response with event metadata
- **THEN** it reads the metadata directly from the mutation result DTO

### Requirement: External mutation behavior remains unchanged
Removing transparent dereferencing SHALL NOT change the GraphQL schema,
mutation response format, or event-recording behavior.

#### Scenario: Execute an existing mutation
- **WHEN** a client executes an existing GraphQL mutation
- **THEN** the schema, response data, and recorded events match the behavior
  before explicit mutation result access was required
