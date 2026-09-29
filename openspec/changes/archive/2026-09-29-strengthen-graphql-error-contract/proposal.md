## Why

GraphQL resolvers currently return `PresentationalError` values through async-graphql's generic display conversion, bypassing the existing public error-code and sanitization logic. Several E2E tests only check that an `errors` key exists, allowing schema-validation failures to masquerade as the intended business errors.

## What Changes

- Route query, mutation, and loader failures through the shared GraphQL error conversion so public messages and `extensions.code` are stable and internal details remain private.
- Exercise validation, not-found, conflict, unexpected, infrastructure, and loader failures through the production schema and assert message, code, path, and sanitization behavior.
- Strengthen E2E error assertions to require a non-empty error array with the expected business code and resolver path.
- Correct the invalid import rollback selection set and audit existing import, create, update, delete, restore, and undo error cases so they fail for the intended reason.
- Add HTTP E2E coverage for representative validation and conflict errors, including successful control inputs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `api-error-contract`: Require the public error contract to hold through real resolver and loader execution, and distinguish business failures from GraphQL parsing or schema-validation failures in E2E tests.

## Impact

The shared presentation error conversion, GraphQL schema tests, loader integration tests, E2E helpers, and existing GraphQL error-path tests are affected. The GraphQL schema remains backward compatible; responses that previously omitted `extensions.code` or exposed internal text are corrected to match the documented contract.
