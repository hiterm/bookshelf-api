## 1. Database Error Classification

- [x] 1.1 Add exact-constraint, operation-context classification for author-name uniqueness violations with infrastructure fallback
- [x] 1.2 Add unit and database-backed tests for conflict classification and unknown-error fallback

## 2. Bulk Book Update Accuracy

- [x] 2.1 Identify an actually missing or cross-tenant input Book ID inside the update transaction
- [x] 2.2 Add repository tests for a missing later item, cross-tenant input, successful updates, and rollback/history behavior

## 3. GraphQL Error Contract

- [x] 3.1 Add centralized async-graphql error extension conversion with stable public codes and sanitized messages
- [x] 3.2 Retain and trace internal error details at the presentation boundary without exposing them
- [x] 3.3 Add conversion tests for NotFound, Validation, Conflict, Infrastructure, and Unexpected errors

## 4. Regression Validation

- [x] 4.1 Verify single update/delete, associated-book conflict, validation, import/bulk paths, revision recording, and transaction rollback regressions
- [x] 4.2 Run formatting, clippy, and the complete locked test suite

## 5. OpenSpec Completion

- [x] 5.1 Mark implementation tasks complete and validate the OpenSpec change
- [ ] 5.2 Sync delta specs and archive the completed change
