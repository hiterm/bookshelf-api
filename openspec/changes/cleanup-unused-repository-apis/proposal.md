## Why

Issue #367 R4 found repository inputs and names that describe obsolete behavior. Removing confirmed dead paths makes the Operation/Revision contract easier to follow without changing public behavior.

## What Changes

- Remove the ignored `DeleteAuthorExtra` argument and the unused singular author lookup/create method.
- Correct stale restore documentation and event-oriented local names for revision numbers.
- Update focused tests to assert merge and delete behavior through the remaining contract.

## Capabilities

### New Capabilities

- `repository-contract-hygiene`: Internal repository interfaces expose only inputs used by current mutation paths.

### Modified Capabilities

None.

## Impact

Author repository trait, PostgreSQL implementation, author/book interactors, focused tests, and current source comments. No GraphQL schema, database schema, or historical artifacts change.
