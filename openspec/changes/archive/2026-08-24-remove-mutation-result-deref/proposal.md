## Why

`MutationResultDto<T>` and `SingleEventMutationResultDto<T>` combine a mutation
value with event metadata; they are result containers rather than transparent
wrappers around `T`. Their `Deref<Target = T>` implementations obscure whether
a field belongs to the result DTO or its `value`, even though major presentation
call sites already access `.value` explicitly.

## What Changes

- Remove `Deref` from `MutationResultDto<T>`.
- Remove `Deref` from `SingleEventMutationResultDto<T>`.
- Require callers to access inner DTO fields explicitly through `.value`.
- Preserve the DTO fields, GraphQL schema and response behavior, and mutation
  event-recording behavior.
- Do not rename either result DTO or its `value` field.

## Capabilities

### New Capabilities

- `explicit-mutation-result-access`: Defines mutation result DTOs as explicit
  containers for values and event metadata rather than transparent wrappers.

### Modified Capabilities

- None.

## Impact

- Affects `src/use_case/dto/mutation.rs` and any compile-time callers that rely
  on implicit dereferencing of mutation result DTOs.
- Does not change GraphQL schema, API responses, dependencies, database schema,
  or mutation/event-recording design.
