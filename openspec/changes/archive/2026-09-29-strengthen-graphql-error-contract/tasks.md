## 1. GraphQL Error Boundary

- [x] 1.1 Add a shared conversion from `PresentationalError` to a sanitized async-graphql error with a stable code
- [x] 1.2 Route all query and mutation use-case failures through the shared conversion
- [x] 1.3 Route nested DataLoader failures through the shared conversion

## 2. Schema-Level Verification

- [x] 2.1 Add production-schema tests for validation, not-found, conflict, unexpected, and infrastructure errors
- [x] 2.2 Add a production-schema test for a loader-originated failure and verify its nested path

## 3. E2E Error Verification

- [x] 3.1 Strengthen the shared E2E error assertion to require a non-empty array, expected code, and expected path
- [x] 3.2 Correct the invalid import rollback selection and assert the intended validation failure
- [x] 3.3 Audit import, create, update, delete, restore, and undo error cases and give each a structured expectation
- [x] 3.4 Add or extend representative validation and conflict HTTP cases with successful controls

## 4. Validation

- [x] 4.1 Run focused unit and E2E tests for the changed error paths
- [x] 4.2 Run formatting, clippy, and the full locked test suite
