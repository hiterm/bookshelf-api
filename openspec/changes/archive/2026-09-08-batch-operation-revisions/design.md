## Context

Operation changes are already loaded per request through Book and Author change
DataLoaders. Their `beforeRevision` and `afterRevision` resolvers bypass that
pattern and call the single-Revision use case independently, producing up to two
Revision SQL queries per change. The frontend selects both fields for both entity
types in its existing Operation detail query.

The baseline is backend `3fe0ee7e51793694f9119ace8be03a3f1d018e58`
and frontend `67a1c979f4a377d2e8cdd7c40f2540727a026778`. The frontend query at
`src/graphql/operation.graphql` selects both before and after snapshots for both
Book and Author changes, including every currently exposed Revision field.

Revision rows are identified by `(user_id, entity_id, revision_number)`. Book
Revision author IDs are stored in `book_revision_author` and are currently
returned in deterministic author-ID order. Existing single-Revision and history
list APIs have callers outside Operation detail and remain supported.

## Goals / Non-Goals

**Goals:**

- Resolve all selected Book Revisions in one Book batch and all selected Author
  Revisions in one Author batch per GraphQL request dispatch cycle.
- Preserve domain validation, tenant isolation, missing/null behavior, Book
  Revision author membership and ordering, and the public GraphQL schema.
- Make empty and duplicate input behavior explicit and regression-tested.
- Provide a testable boundary showing one batch use-case/repository call for many
  loader keys.

**Non-Goals:**

- Change the frontend query, UI rendering, GraphQL schema, or response shape.
- Add pagination, virtual scrolling, global caching, or unrelated Operation
  performance work such as undo eligibility optimization.
- Remove the existing single-Revision APIs or add a database migration without
  query-plan evidence.

## Decisions

1. **Use explicit composite key types at each boundary.** Repository keys pair
   domain entity IDs with `RevisionNumber`; GraphQL loader keys pair a String ID
   with an `i32` Revision number. Both are hashable value types. This avoids
   ambiguous string concatenation while keeping GraphQL/DataLoader types out of
   the domain layer. Tuple aliases were considered, but named types make Book and
   Author keys impossible to mix accidentally.

2. **Send paired arrays through `UNNEST` and join on both key columns.** Each
   repository method deduplicates the typed keys, binds parallel UUID and integer
   arrays, and joins those rows to the owner-scoped Revision table in one query.
   This uses the existing `(user_id, entity_id, revision_number)` primary key and
   avoids dynamic SQL. Looping single lookups is rejected because it retains the
   N+1 behavior; independent `ANY` predicates are rejected because they create a
   Cartesian combination rather than preserving requested pairs.

3. **Retain the existing correlated author aggregation for Book Revisions.** The
   batch query applies the same ordered `array_agg` expression per selected
   Revision, preserving current meaning and ordering. A broader join/grouping
   rewrite is unnecessary for this focused change.

4. **Register two request-scoped DataLoaders in the GraphQL handler.** Both before
   and after resolvers for an entity type use the same loader instance. A null
   Revision number returns null before calling the loader. Claims remain captured
   in each loader, so caches cannot cross users or requests.

5. **Measure batching at stable boundaries.** Unit tests assert that many loader
   keys cause one batch use-case call and that use cases call only batch repository
   methods. Repository integration tests exercise the actual one-statement batch
   SQL, including tenant isolation and Book author IDs. GraphQL tests verify the
   existing response contract and shared before/after loader behavior. Exact
   whole-request SQL counts are not asserted because unrelated resolver queries
   may evolve.

## Risks / Trade-offs

- **[PostgreSQL cannot infer an empty array type]** → Return an empty map before
  issuing SQL and cast bound arrays explicitly in `UNNEST`.
- **[Parallel key arrays could become misaligned]** → Build both arrays in one
  pass from the same deduplicated key sequence and cover mixed pairs in tests.
- **[DataLoader dispatch can form more than one batch if resolver scheduling
  changes]** → Assert calls at the loader boundary and require query count to be
  independent of change count rather than hard-coding a whole-request count.
- **[Adding hashability to an ID value object broadens its supported use]** → Hash
  only the immutable UUID identity, consistent with equality.

## Migration Plan

Deploy as an application-only change. No schema or data migration is required.
Rollback restores the former resolvers and leaves all persisted data untouched.

## Open Questions

None.
