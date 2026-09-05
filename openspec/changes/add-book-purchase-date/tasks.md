## 1. Persistence and Domain

- [ ] 1.1 Add nullable purchase-date migration columns and migration coverage
- [ ] 1.2 Add purchase date to book domain state, updates, and domain tests
- [ ] 1.3 Thread purchase date through repository rows, SQL, fixtures, and tests

## 2. Use Cases and History

- [ ] 2.1 Extend create, update, import, and preview DTOs and interactors
- [ ] 2.2 Extend revision append, history, restore, and undo paths and tests

## 3. GraphQL API

- [ ] 3.1 Expose nullable `Date` fields on all book inputs and objects
- [ ] 3.2 Regenerate `schema.graphql` and add GraphQL behavior coverage

## 4. Verification and Delivery

- [ ] 4.1 Run formatting, clippy, locked tests, migration tests, and E2E tests
- [ ] 4.2 Inspect the final diff and confirm OpenSpec alignment
- [ ] 4.3 Commit source separately, push, open a PR, and verify CI and review
- [ ] 4.4 Archive the completed OpenSpec change in a separate commit
