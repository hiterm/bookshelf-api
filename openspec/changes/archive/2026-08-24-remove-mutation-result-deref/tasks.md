## 1. Usage Audit

- [x] 1.1 Search both mutation result DTOs, their `Deref` implementations, and
  all named type aliases for implicit inner-value field access
- [x] 1.2 Confirm existing unit coverage for create/update book,
  create/update author, delete, restore, import, and merge paths

## 2. Explicit Result Access

- [x] 2.1 Remove the `MutationResultDto<T>` `Deref` implementation
- [x] 2.2 Remove the `SingleEventMutationResultDto<T>` `Deref` implementation
- [x] 2.3 Convert any confirmed implicit inner-value access to explicit `.value`
  access without changing event metadata access

## 3. Verification

- [x] 3.1 Run `cargo fmt --check`
- [x] 3.2 Run `cargo clippy --all-targets --locked -- -D warnings`
- [x] 3.3 Run `cargo test --locked` and confirm the audited unit paths pass
- [x] 3.4 Run the repository's existing E2E suite unchanged and confirm the
  GraphQL API behavior remains stable
- [x] 3.5 Review the complete diff for unrelated code, schema, mutation return,
  event-recording, or unnecessary test changes

## 4. Delivery

- [x] 4.1 Commit the implementation and OpenSpec change
- [x] 4.2 Push the feature branch and open a PR against `main`
- [x] 4.3 Confirm all PR CI checks pass and perform the final PR diff and
  OpenSpec consistency review
