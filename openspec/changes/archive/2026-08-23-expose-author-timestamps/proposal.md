## Why

Books expose their creation and update timestamps through GraphQL, while authors only retain equivalent timestamps in the database and event history. Exposing author timestamps removes this API inconsistency and lets clients order or display authors using their lifecycle metadata.

## What Changes

- Add `createdAt` and `updatedAt` fields to the GraphQL `Author` type.
- Carry author timestamps from persistence through the domain and use-case layers.
- Cover timestamp persistence and GraphQL serialization with unit tests.

## Capabilities

### New Capabilities

- `author-timestamps`: Expose persisted author creation and update times through every GraphQL author representation.

### Modified Capabilities

None.

## Impact

The author domain entity, repository mapping, use-case DTO, GraphQL object, schema tests, and related fixtures are affected. The database schema and existing GraphQL fields remain unchanged, so this is an additive API change.
