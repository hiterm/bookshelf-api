## 1. Batch Event Queries

- [x] 1.1 Add user-scoped multi-event-set queries to book and author event repositories and database implementations
- [x] 1.2 Add repository tests for multiple IDs, mixed ownership, missing events, and empty ID lists
- [x] 1.3 Add event-query use-case batch operations and unit tests for conversion, empty input, and empty results

## 2. Use-Case Event Set Model

- [x] 2.1 Remove `EventSetDetailDto` and make single-event-set lookup return the scalar-only `EventSetDto`
- [x] 2.2 Replace eager-loading detail tests with lookup validation, found, and not-found unit tests

## 3. GraphQL Loading and Model

- [x] 3.1 Add request-scoped book-events-by-event-set and author-events-by-event-set DataLoaders with empty-list grouping
- [x] 3.2 Replace `EventSetEntry` and `EventSetDetail` with one complex `EventSet` object whose nested fields use the DataLoaders
- [x] 3.3 Update query resolvers and dependency injection so list and single-item queries return `EventSet` without eager loading
- [x] 3.4 Add GraphQL schema/resolver tests for the unified type, nested detail/list results, empty lists, and batched calls
- [x] 3.5 Regenerate `schema.graphql` and verify removed type names, field types, and nullability

## 4. API Validation

- [x] 4.1 Run OpenSpec validation and confirm the implementation matches the event-set query-model scenarios
- [x] 4.2 Run formatting, clippy, locked unit/integration tests, and the existing event-history E2E regression suite

## 5. Frontend Schema Follow-Up

- [x] 5.1 Update the `bookshelf` local GraphQL schema and regenerate GraphQL client types
- [x] 5.2 Verify EventSet list/detail components and operations require no behavior changes, making only necessary type follow-ups
- [x] 5.3 Run frontend generate, lint, unit, typecheck, and relevant event-history E2E regression suites

## 6. Delivery

- [x] 6.1 Commit logical API changes after mandatory checks, push, and open a PR documenting the breaking rename, lazy loading, batching, and test results
- [x] 6.2 Confirm frontend schema generation produces no tracked changes, so no empty frontend commit or PR is required
- [ ] 6.3 Monitor all required CI checks on both PRs and fix failures until both are green
