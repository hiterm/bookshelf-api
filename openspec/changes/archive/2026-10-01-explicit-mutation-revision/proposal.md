## Why

Single-entity mutation responses currently read a mutable transaction slot instead of the repository result. This couples response metadata to write order and lets mocks conceal incorrect revision selection (Issue #367 R1).

## What Changes

- Return validated `RevisionNumber` values from Book and Author create/update.
- Return restored entities together with their newly recorded revision numbers.
- Build response metadata directly from repository results and remove transaction revision state.
- Verify distinct revision results in unit tests and persisted history in database tests.

## Capabilities

### New Capabilities

- `mutation-revision-results`: Explicit per-entity revision results across repository and use-case boundaries.

### Modified Capabilities

None.

## Impact

Domain repository contracts, PostgreSQL history recording, transaction context, use-case mocks and tests, and current architecture documentation. GraphQL schema and database schema remain unchanged; bulk operations retain their existing result contracts.
