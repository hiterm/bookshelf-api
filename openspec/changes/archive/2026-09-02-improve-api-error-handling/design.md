## Context

All `sqlx::Error` values currently fall through a global conversion to `DomainError::InfrastructureError`. The database has one user-visible uniqueness rule, `author_user_id_name_unique`; other primary, foreign-key, and history constraints primarily protect identity, tenant ownership, and event-recording invariants. `BookRepository::update_all` detects a row-count mismatch but reports the first input book regardless of which row was absent. Presentation errors currently derive their GraphQL message from `Display`, exposing `user_id` and internal error text, and provide no stable code.

Mutations must remain inside the existing transaction and Operation/Revision recording boundary. The established DomainError → UseCaseError → PresentationalError layering and resolver result signatures remain in place.

## Goals / Non-Goals

**Goals:**

- Convert violations of the named author uniqueness constraint in author create/update operations to `DomainError::Conflict`.
- Preserve infrastructure classification for unknown SQL errors and constraints whose violation indicates an invariant or infrastructure defect.
- Identify a genuinely absent or cross-tenant book during a bulk update before any dependent history work proceeds.
- Serialize presentation errors with safe messages and stable GraphQL codes at one shared conversion boundary.
- Log internal failures once at that boundary while retaining their underlying diagnostic causes.

**Non-Goals:**

- Redesigning the error enums or introducing a bulk-specific error type.
- Mapping every PostgreSQL SQLSTATE or constraint to a business error.
- Changing GraphQL HTTP statuses, Claims extraction, or JWKS handling.
- Changing database schemas or transaction/event-recording semantics.

## Decisions

1. Add a small repository-local classifier for `author_user_id_name_unique`, and invoke it only around author `create` and `update` statements. The classifier checks both PostgreSQL unique-violation SQLSTATE `23505` and the exact constraint name, then returns a useful author-name conflict. All other errors use the existing `From<sqlx::Error>` fallback. A global SQLSTATE-only mapping was rejected because unrelated identity and history constraints do not represent the same API conflict.

2. When `update_all` affects fewer rows than inputs, query the same transaction for input IDs owned by the transaction user and compare those IDs with the ordered input. Return the first input ID that is absent from the owned set. Keeping input order makes multiple-missing behavior deterministic without adding a new error type. The check stays after the scoped set-based update, so rollback preserves atomicity; no relationship or revision writes occur after mismatch detection.

3. Implement `async_graphql::ErrorExtensions` for `PresentationalError`. The implementation will construct a public message per variant and add exactly one `code`: `NOT_FOUND`, `VALIDATION_ERROR`, `CONFLICT`, or `INTERNAL_ERROR`. This is the shared conversion used by async-graphql's normal `Result<T, E>` handling, avoiding resolver-specific code.

4. Construct NotFound presentation messages from `entity_type` and `entity_id`, not `UseCaseError::to_string()`, so `user_id` remains available internally but never enters the GraphQL message. Validation and conflict messages remain actionable. Other and unexpected errors use `Internal server error`; the underlying error/message is retained in `PresentationalError` and recorded with tracing when converted to GraphQL.

5. Test the classifier as a pure unit where possible and through database-backed author mutation tests for the actual named constraint. Test presentation conversion directly by inspecting async-graphql error extensions, avoiding brittle complete-message assertions. Extend bulk repository tests for a missing second item, cross-tenant scope, and normal success/rollback behavior.

## Risks / Trade-offs

- [A newly added user-visible constraint initially falls back to internal error] → Require explicit repository-context classification and tests when introducing such a constraint.
- [The mismatch verification adds one SQL round trip only on failed bulk updates] → Accept the failure-path cost to preserve bounded set-based success behavior and accurate reporting.
- [Logging during GraphQL conversion could repeat an upstream log] → Keep error logging centralized at the presentation boundary; existing layers do not log these errors.
- [Concurrent deletion could alter mismatch results] → The check uses the same transaction and mutation flow; existing transaction isolation and rollback semantics remain unchanged.

## Migration Plan

No database migration is required. Deploy the application change normally; clients may begin consuming `errors[].extensions.code` immediately. Rollback restores prior error serialization without changing stored data.

## Open Questions

None.
