## 1. Explicit repository results

- [x] 1.1 Add typed revision results and update repository/history recording contracts.
- [x] 1.2 Remove transaction revision state and consume explicit results in use cases.

## 2. Verification and documentation

- [x] 2.1 Strengthen unit tests with distinct revision values and preserve failure coverage.
- [x] 2.2 Verify create/update/restore result consistency in database tests and assess E2E with the user.
- [x] 2.3 Update current architecture documentation and run formatting, lint, tests, and schema checks.

## 3. Finalization

- [x] 3.1 Sync delta specifications and archive the completed change.

Validation: fmt, clippy, 176 unit tests, 235 database-feature tests (one existing ignored test), existing E2E suite, schema comparison, and strict OpenSpec validation pass. Existing E2E coverage is retained pending any user preference for augmentation.
