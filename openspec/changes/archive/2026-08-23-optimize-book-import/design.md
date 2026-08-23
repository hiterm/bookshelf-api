## Context

The current `BookCommandInteractor::import` owns the correct transaction and
event-set boundary, but calls `find_or_create_by_name` for each unique author
and `create` for each book. Each repository method executes several SQL
statements, making database round trips proportional to input size. PostgreSQL
already enforces author uniqueness by `(user_id, name)`, and the repository
adapters already use array binding and `UNNEST` elsewhere.

The GraphQL contract, `MAX_BOOK_BATCH = 1000`, entity snapshots, event
semantics, and existing single-entity mutation paths are constraints. Every
write must remain inside the transaction opened with
`EventSetOperation::ImportBooks`.

## Goals / Non-Goals

**Goals:**

- Make the import write path use a bounded, nearly constant number of SQL
  statements independent of book and unique-author counts.
- Preserve author/book rows, relationship rows, event snapshots, the shared
  event set, atomic rollback, validation, and response behavior.
- Keep deduplication and entity assembly in the interactor while keeping SQL,
  conflict handling, and event persistence in PostgreSQL adapters.

**Non-Goals:**

- Changing the GraphQL schema, import limits, or single-entity repository paths.
- Changing database schemas or adding dependencies.
- Adding timing-sensitive CI assertions or permanent benchmark infrastructure.
- Optimizing the existing `event_set` insertion, which already occurs once.

## Decisions

### Add explicit bulk repository methods

`AuthorRepository` gains `find_or_create_by_names`, returning all requested
name-to-ID mappings, and `BookRepository` gains `create_all`. The existing
single-entity methods remain intact. Explicit APIs prevent the interactor from
depending on PostgreSQL details and prevent a misleading default loop over
single writes.

Alternative: reuse single methods concurrently. This still issues O(N + A)
queries, complicates transaction borrowing, and does not meet the goal.

### Deduplicate author names before repository access

The interactor collects unique `AuthorName` values across the import before
opening repository loops, invokes the author bulk method once, and removes
duplicate author IDs within each book while preserving first-seen order.
In-memory loops are allowed; no awaited repository call occurs inside them.

### Resolve authors with conflict-aware bulk SQL

The PostgreSQL adapter performs one `INSERT ... SELECT FROM UNNEST ... ON
CONFLICT (user_id, name) DO NOTHING RETURNING ...`. Returned rows identify only
authors created by this import and feed one bulk `author_event` insert. One
subsequent `SELECT ... WHERE name = ANY(...)` resolves IDs for both existing
and newly created authors.

Alternative: select then insert. It requires extra round trips and introduces a
check-then-insert race. The unique constraint is the concurrency authority.

### Persist books and their events in set-based statements

`create_all` builds typed arrays from all books and executes at most four
statements: book insert, optional `book_author` insert, book-event insert with
`RETURNING (event_id, book_id)`, and optional `book_event_author` insert. Every
book event uses the transaction's event-set ID and preserves the existing
snapshot representation. Flattening relationships and matching returned event
IDs happen in memory.

`UNNEST` is preferred over dynamic multi-row SQL because the repository already
uses typed PostgreSQL arrays, input is capped, and statement shape remains
constant.

### Keep one transaction and one event set

The interactor begins exactly once with `ImportBooks`, calls each bulk method
once, and commits once. Repositories derive `user_id` and `event_set_id` from
that transaction. Any statement failure returns before commit, so dropping the
transaction rolls back all author, book, relationship, and event writes.

## Risks / Trade-offs

- [Large parallel arrays can be misaligned] → Construct arrays together from
  the same iteration and cover multi-author snapshots with database tests.
- [Bulk `RETURNING` order is unspecified] → Build maps keyed by author name
  and book ID; never rely on row order.
- [Empty arrays can cause PostgreSQL type ambiguity] → Skip optional
  relationship/event statements when their flattened inputs are empty and bind
  explicit array types elsewhere.
- [Concurrent imports can race on author creation] → Use the unique
  constraint plus `ON CONFLICT DO NOTHING`, then resolve every input name in one
  post-insert select.
- [A missing resolved author could silently corrupt assembly] → Treat an
  incomplete resolution result as a domain/infrastructure error before books
  are written.

## Migration Plan

No database or API migration is required. Deploy the repository and interactor
changes together. Rollback is the prior application version because both
versions use the same schema and external contract.

## Open Questions

None.
