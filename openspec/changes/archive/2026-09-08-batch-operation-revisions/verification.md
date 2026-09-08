## Baseline

- Backend start: `3fe0ee7e51793694f9119ace8be03a3f1d018e58`
- Frontend start: `67a1c979f4a377d2e8cdd7c40f2540727a026778`
- Frontend query: `src/graphql/operation.graphql` selects both before and after
  Revisions for Book and Author changes. No frontend change is required.
- Previous path: each non-null Revision field called `book_revision` or
  `author_revision`, and each call issued one SQL statement. For an import with
  25 created Books and 25 created Authors selecting `afterRevision`, the path
  therefore issued 50 Revision SQL statements. Selecting both sides of updated
  changes could issue twice the number of changes.

## Result

- Each loader test passes multiple composite keys and asserts exactly one batch
  use-case call. The use-case test asserts exactly one batch repository call.
- Each PostgreSQL batch repository method contains one `UNNEST`-join query and
  returns before SQL for empty keys. Thus the same 25 Book plus 25 Author example
  issues one Book Revision query and one Author Revision query per DataLoader
  dispatch: 2 Revision SQL statements instead of 50.
- The DB-backed library suite completed 221 tests successfully (1 ignored),
  including paired keys, duplicates, missing keys, cross-owner same UUIDs, and
  ordered Book Revision author IDs.
- The updated 25-entry import E2E, which selects full Book after Revisions and
  Author after Revisions, completed successfully in 4.45 seconds. This duration
  includes user setup, the import mutation, verification requests, and cleanup;
  it is not an isolated Operation response benchmark.
- The focused Operation history and merge E2E tests completed successfully (3
  tests total), covering create, before/after, mixed Book/Author changes, null
  before Revision, tenant isolation, and response compatibility.

## Assessment

An isolated before/after response-time comparison was not retained because the
baseline server and current server would require separately provisioned identical
databases and the request-boundary call-count regression is deterministic. The
query-count improvement is structural and covered at both loader and repository
boundaries.

Both Revision tables already have primary keys beginning with
`(user_id, entity_id, revision_number)` plus matching ordered indexes. The batch
queries join on the full primary key, so no migration or additional index is
justified. The GraphQL schema and frontend query are unchanged; frontend
pagination or virtual scrolling remains outside this change.
