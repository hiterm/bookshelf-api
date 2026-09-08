## 1. Baseline and Contract

- [x] 1.1 Record backend/frontend starting SHAs, confirm the frontend Operation query and document the pre-change Revision retrieval path
- [x] 1.2 Commit the completed OpenSpec proposal, design, delta specification, and implementation tasks separately from source changes

## 2. Repository and Use Case

- [ ] 2.1 Add typed Book and Author Revision batch keys and owner-scoped repository APIs that issue no SQL for empty inputs
- [ ] 2.2 Implement paired-key PostgreSQL batch queries with duplicate, missing, cross-owner, mixed-key, and Book author-ID integration coverage
- [ ] 2.3 Expose validated batch Revision lookups through HistoryQueryUseCase and test one batch repository call with correctly mapped DTO results

## 3. GraphQL Batch Loading

- [ ] 3.1 Add and unit-test request-scoped BookRevisionLoader and AuthorRevisionLoader composite-key batching
- [ ] 3.2 Register both Revision loaders and route all four Operation change Revision resolvers through the shared per-entity loaders while bypassing null keys
- [ ] 3.3 Add GraphQL regression coverage for create/update, mixed entity types, missing sides, response compatibility, and many changes without per-field single lookups

## 4. Verification and Delivery

- [ ] 4.1 Record before/after query-count evidence and representative timing, and assess whether any index or frontend change is justified
- [ ] 4.2 Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`, plus relevant integration/E2E tests
- [ ] 4.3 Commit implementation and test changes at logical breakpoints, sync the delta spec, archive the completed OpenSpec change, and commit the archive separately
- [ ] 4.4 Push the branch, open a PR describing cause, batching design, compatibility, tests, and performance, then verify CI and request `@coderabbitai review`
