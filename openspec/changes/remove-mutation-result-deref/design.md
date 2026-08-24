## Context

Mutation use cases return `MutationResultDto<T>` or
`SingleEventMutationResultDto<T>`, which pair a `value` with `event_set_id` and,
for single-event mutations, `event_id`. Both types currently implement `Deref`
to `T`, allowing implicit inner-field access even though presentation code
primarily uses `.value` already.

This is an internal type-boundary refactor. The DTO layouts, GraphQL conversion
paths, transaction boundaries, and event-recording behavior remain unchanged.

## Goals / Non-Goals

**Goals:**

- Make all inner mutation value access explicit through `.value`.
- Remove both generic `Deref<Target = T>` implementations.
- Preserve compilation and all existing unit and E2E behavior.

**Non-Goals:**

- Rename either result DTO or its `value` field.
- Change mutation payload GraphQL schema or response shape.
- Change mutation orchestration, transaction management, or event recording.
- Add a runtime test that attempts to prove the absence of a trait
  implementation.

## Decisions

- Remove only the two `Deref` implementations and update call sites only where
  compilation or source inspection identifies implicit inner-value access. This
  keeps the diff focused and preserves the result structures.
- Access `value` fields as `result.value.<field>` while continuing to read
  `event_set_id` and `event_id` directly from the result DTO. This makes field
  ownership visible at each use site.
- Rely on compilation and existing unit coverage for create/update book,
  create/update author, delete, restore, import, and merge paths. A dedicated
  runtime test cannot meaningfully assert that `Deref` is absent; new unit tests
  are added only if the coverage audit finds a functional gap.
- Run existing E2E tests unchanged because there is no new endpoint or GraphQL
  schema change. Their unchanged success verifies the external contract.

An alternative was to retain `Deref` for convenience, but that preserves the
ambiguous field access this change is intended to eliminate. Introducing named
accessor methods was also rejected because the public `value` field already
provides an explicit, conventional access path.

## Risks / Trade-offs

- [Risk] A call site may rely on auto-dereferencing in a non-obvious expression.
  → Mitigation: search all result types and aliases, then compile all targets.
- [Risk] Broad mechanical edits could obscure the intended no-behavior-change
  refactor. → Mitigation: change only confirmed implicit accesses and review the
  complete diff before committing.
- [Trade-off] Call sites become slightly more verbose. → The explicit ownership
  of value and event fields is the intended design benefit.

During implementation, all production and presentation call sites were found
to use explicit result fields already. Existing author and book interactor unit
tests still relied on implicit inner-field access, including vector methods and
indexing for import results, so only those assertions require `.value` updates.
