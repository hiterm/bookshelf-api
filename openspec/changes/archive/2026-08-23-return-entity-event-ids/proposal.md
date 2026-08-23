## Why

Book and Author create/update mutations currently return the logical operation's
`eventSetId`, but clients cannot directly identify the specific recorded entity
snapshot. Returning that event's ID lets clients inspect history and use the
newly recorded snapshot as a restore source without an additional lookup.

## What Changes

- Add a required `eventId` field to `BookMutationPayload` and
  `AuthorMutationPayload` for create and update mutations.
- Define `eventId` as the newly recorded Book or Author create/update event,
  distinct from the operation-wide `eventSetId`.
- Propagate the inserted event ID through repository and use-case mutation
  results while preserving the existing transaction boundary.
- Add unit and E2E coverage linking returned IDs to event history and restore
  inputs.
- Document that mutations which can produce zero or multiple entity events do
  not return a single `eventId`.

## Capabilities

### New Capabilities

- `entity-mutation-event-ids`: Defines how Book and Author create/update
  mutations expose the newly recorded entity event alongside its event set.

### Modified Capabilities

None.

## Impact

This additive GraphQL API change affects Book and Author repositories, mutation
use-case DTOs and traits, GraphQL payload construction, generated
`schema.graphql`, event-recording documentation, and unit/E2E tests. It does not
change delete, restore, import, or user-registration payload semantics, database
schema, or the `TransactionManager` boundary.
