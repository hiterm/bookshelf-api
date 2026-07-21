## Context

The `author` table already stores non-null `created_at` and `updated_at` values, and author event snapshots preserve both values. Unlike `Book`, however, the `Author` domain entity, DTO, and GraphQL object omit them. The repository currently discards the timestamps when reconstructing authors.

## Goals / Non-Goals

**Goals:**

- Preserve persisted author timestamps when loading and mutating authors.
- Expose timestamps as Unix seconds through GraphQL, matching `Book`.
- Keep all existing author queries and mutations source-compatible.

**Non-Goals:**

- Changing the database schema or timestamp precision.
- Adding timestamp filters or ordering to author queries.
- Altering author event history fields.

## Decisions

- Add `created_at` and `updated_at` to the `Author` aggregate and its destructured form. This matches the established `Book` approach and prevents persistence concerns from leaking directly into presentation code. An alternative was a repository-only wrapper, but that would split the entity representation across types.
- Have new authors receive a single application-generated UTC timestamp for both fields, while repository reads retain database values. This follows the `Book` construction pattern and makes newly returned mutation payloads immediately complete.
- Serialize both values as signed 64-bit Unix seconds named `createdAt` and `updatedAt`, matching the existing GraphQL `Book` API.

## Risks / Trade-offs

- [Adding fields changes constructors and fixtures throughout the codebase] → Update all compile-time call sites and add focused conversion tests.
- [Application and database clocks can differ slightly on insert] → Persist the entity timestamps explicitly, as the book repository already does, so the returned and stored values agree.

## Migration Plan

No data migration is required because both columns already exist and are non-null. Deploy the additive GraphQL schema and server changes together; rollback removes the fields without changing stored data.

## Open Questions

None.
