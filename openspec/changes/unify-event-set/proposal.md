## Why

The API currently represents an event set with separate list and detail types even though both types describe the same domain concept. Unifying the public model makes the schema consistent while preserving efficient, selection-driven loading of nested events.

## What Changes

- **BREAKING** Replace the GraphQL `EventSetEntry` and `EventSetDetail` types with one `EventSet` type used by both existing query fields.
- Preserve the existing query field names, response fields, and nullability while resolving `bookEvents` and `authorEvents` lazily.
- Batch nested event loading across event-set IDs with DataLoader to avoid N+1 repository queries.
- Remove `EventSetDetailDto`; use the scalar-only `EventSetDto` for both list and single-item use cases.
- Add user-scoped repository and use-case batch queries for book and author events.
- Keep the domain `EventSet` unchanged.
- Update the `bookshelf` local schema and generated GraphQL types without changing the event-history UI behavior.

## Capabilities

### New Capabilities

- `event-set-query-model`: Defines the unified GraphQL event-set model and selection-driven, batched loading behavior for nested events.

### Modified Capabilities

None.

## Impact

- Affects event query DTOs and use cases, book/author event repositories, database queries, GraphQL loaders, objects, query resolvers, schema tests, and generated SDL in `bookshelf-api`.
- Requires a coordinated schema and generated-type update in `hiterm/bookshelf`.
- Clients that refer directly to the removed GraphQL type names must regenerate or update types; the existing query field response shapes remain compatible.
