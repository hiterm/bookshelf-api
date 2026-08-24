## ADDED Requirements

### Requirement: Unified event-set GraphQL type
The GraphQL schema SHALL expose one `EventSet` type for both the event-set list and single-event-set query and SHALL NOT expose `EventSetEntry` or `EventSetDetail` types.

#### Scenario: Query field types are unified
- **WHEN** a client inspects the GraphQL schema
- **THEN** `eventSets` returns non-null `EventSet` elements and `eventSet` returns a nullable `EventSet`

#### Scenario: Event-set fields remain available
- **WHEN** a client selects scalar, book-event, or author-event fields from either event-set query
- **THEN** the unified type exposes the existing fields with their existing nullability

### Requirement: Selection-driven nested event loading
The system SHALL load book and author events only when their corresponding GraphQL fields are selected.

#### Scenario: Scalar-only list query
- **WHEN** a client requests only scalar fields from `eventSets`
- **THEN** the system does not query book events or author events

#### Scenario: Single event set with nested events
- **WHEN** a client requests `bookEvents` or `authorEvents` from `eventSet`
- **THEN** the system returns the requested events for that event set

#### Scenario: Event set with no nested events
- **WHEN** a selected nested event field has no events for an event set
- **THEN** the system returns an empty list for that field

### Requirement: Batched nested event loading
The system MUST batch each selected nested event kind across all event sets resolved within a GraphQL request and MUST preserve user ownership filtering.

#### Scenario: Multiple event sets request book events
- **WHEN** a client selects `bookEvents` for multiple event sets in one query
- **THEN** the system retrieves book events with one user-scoped batch query rather than one query per event set

#### Scenario: Multiple event sets request author events
- **WHEN** a client selects `authorEvents` for multiple event sets in one query
- **THEN** the system retrieves author events with one user-scoped batch query rather than one query per event set

#### Scenario: Events belong to another user
- **WHEN** a batch query includes event-set IDs associated with another user
- **THEN** the system does not return that user's events

### Requirement: Scalar-only use-case event-set model
The event query use case SHALL return the same scalar-only event-set DTO for list and single-item lookups, while nested events SHALL be obtained through dedicated batch operations.

#### Scenario: Single event-set lookup
- **WHEN** the use case finds an event set by a valid ID
- **THEN** it returns the event-set ID, operation, and creation time without eagerly loading nested events

#### Scenario: Batch operation receives no IDs
- **WHEN** a nested event batch operation receives an empty event-set ID list
- **THEN** it returns an empty result without failing

