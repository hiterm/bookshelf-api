## Why

Operation detail currently batches Book and Author change rows, but resolves each
change's before and after Revision with an individual repository query. Large
imports therefore make Revision query count and response latency grow with the
number of changes.

## What Changes

- Add owner-scoped batch lookup APIs for Book and Author Revisions keyed by
  entity ID and revision number.
- Add request-scoped Revision DataLoaders and use them from all four Operation
  change Revision resolvers.
- Preserve the existing GraphQL fields, nullability, Revision contents, missing
  Revision behavior, and tenant isolation.
- Add repository, use-case, loader, and GraphQL regression coverage proving that
  Revision lookup is batched rather than repeated per change.
- Record the existing and improved Revision retrieval paths and performance
  characteristics using reproducible tests or measurements.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `operation-history`: Require before and after Revisions selected through
  Operation changes to be resolved with owner-scoped batch loading whose query
  count does not grow with the number of changes.

## Impact

The change affects the history repository and its PostgreSQL queries, history
use-case APIs, GraphQL loaders and Operation change resolvers, request-scoped
loader registration, and related unit/integration/E2E tests. The public GraphQL
schema and the frontend Operation query remain unchanged, and no database
migration or new dependency is expected.
