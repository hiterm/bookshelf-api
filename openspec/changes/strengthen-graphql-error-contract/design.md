## Context

`PresentationalError` implements async-graphql's `ErrorExtensions`, but resolver signatures return `Result<T, PresentationalError>`. Async-graphql therefore applies its blanket `Display` conversion instead of calling `extend()`, dropping `extensions.code` and exposing the display text of internal errors. DataLoader errors face the same issue when propagated from complex-object resolvers. Separately, E2E helpers accept any `errors` member, including empty arrays and schema-validation failures with no business code or resolver path.

## Goals / Non-Goals

**Goals:**

- Make every use-case failure returned by query, mutation, and complex-object loader resolvers use one shared conversion to `async_graphql::Error`.
- Preserve actionable validation, not-found, and conflict messages while sanitizing unexpected and infrastructure details.
- Verify the production schema execution path, including loader failures.
- Make existing business-error E2E tests assert the intended code and path and repair the invalid import selection.

**Non-Goals:**

- Change the use-case error taxonomy or GraphQL field names.
- Inject database outages into HTTP E2E tests.
- Add API endpoints or alter mutation/event-recording behavior.
- Address the unrelated C-N items in Issue #364.

## Decisions

### Use async-graphql's custom error conversion boundary

The async-graphql dependency will enable `custom-error-conversion`, and `PresentationalError` will implement `From<PresentationalError> for async_graphql::Error` by delegating to the shared public conversion. Query and mutation resolvers can retain their typed results while async-graphql attaches path and source location after applying the application-defined conversion.

An alternative was to add `map_err` at every resolver call site. That repeats a security-sensitive boundary throughout query and mutation code and allows future resolvers to omit it. Without `custom-error-conversion`, relying on `Result<T, PresentationalError>` plus the `ErrorExtensions` implementation invokes async-graphql's blanket `Display` conversion and does not call `extend()`.

### Keep DataLoader errors cloneable and convert at complex-object resolvers

Loader implementations retain `PresentationalError` as their associated error because DataLoader requires cloneable errors and direct loader tests benefit from the typed error. Propagation from a complex-object resolver uses the same application-defined `From` conversion as top-level resolvers.

An alternative was to store `async_graphql::Error` as the loader error, which would mix transport representation into batching logic and complicate the existing direct loader tests.

### Test behavior at both schema and HTTP boundaries

Mock-backed production-schema tests will cover each error category and loader propagation without external failure injection. HTTP E2E tests will cover representative validation and conflict responses using real inputs and will use successful control inputs to demonstrate that the resolver path is reachable.

### Require structured E2E error expectations

The shared E2E assertion will require a non-empty error array, exact `extensions.code`, and an expected resolver path. Existing business-error call sites in import, create/update/delete, restore, and undo tests will provide explicit expectations. Parsing and schema-validation errors are therefore not interchangeable with business failures.

## Risks / Trade-offs

- [Risk] Enabling custom conversion removes async-graphql's generic conversion for arbitrary display errors. → Keep GraphQL resolvers on the existing `PresentationalError` boundary and let compilation reject any new unclassified resolver error type.
- [Risk] Strengthened E2E assertions reveal additional incorrect expectations. → Audit every existing helper call and fix queries or expected categories rather than weakening the helper.
- [Risk] Conflict E2E setup may depend on database constraint behavior. → Use the supported duplicate-author-name operation and a unique run identifier, plus a successful distinct-name control.
