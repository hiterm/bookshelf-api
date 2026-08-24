## Context

`mergeAuthor` locks Books before Authors to remain compatible with the lock
order used by concurrent Book updates. It then rewrites each Book through the
domain entity and calls the single-entity repository update once per Book.
That repository call performs separate statements for the Book row,
`book_author`, `book_event`, and `book_event_author`, so the merge's SQL round
trips scale linearly with the number of affected Books.

The existing `create_all()` implementation demonstrates the project's UNNEST
pattern for bulk Book and event persistence. All writes must remain within the
use-case-owned transaction and use its user and event-set identifiers.

## Goals / Non-Goals

**Goals:**

- Persist multiple updated Book aggregates with set-based SQL and a nearly
  fixed number of database round trips.
- Produce the same live Book state and one equivalent update event per Book.
- Preserve tenant isolation, transaction atomicity, and the Book-to-Author lock
  ordering used by `mergeAuthor`.
- Treat an empty input as a successful no-op.

**Non-Goals:**

- Changing Author merge rules, Author merge events, Book event semantics, the
  GraphQL API, or `eventSetId` handling.
- Adding a merge-specific repository method.
- Changing the `book_author` primary key or introducing repository-level
  uniqueness rules.
- Replacing `Book::update()` with infrastructure-side domain mutation.

## Decisions

### Add a general `BookRepository::update_all()` operation

The trait will accept a transaction and a slice of fully updated Book entities.
`mergeAuthor` will mutate all fetched entities with `Book::update()` before one
call to this method. The existing `update()` remains the path for ordinary
single-Book mutations. A merge-specific operation was rejected because it
would couple persistence to one use case and blur the domain/persistence
boundary.

### Use fixed-count, set-based statements

Rust will flatten Book fields and final `(book_id, author_id)` relationships
into arrays. PostgreSQL UNNEST inputs will drive:

1. one user-scoped Book UPDATE that leaves `created_at` unchanged;
2. one user-scoped DELETE of relationships absent from the final state;
3. one INSERT of final relationships with `ON CONFLICT DO NOTHING`;
4. one bulk INSERT of update event snapshots returning `(event_id, book_id)`;
5. one bulk INSERT of event-author snapshots.

Empty relationship arrays will still allow the DELETE to remove every author
from a targeted Book, while relationship and event-author INSERTs may be
skipped when there is nothing to insert. Loops may prepare arrays but must not
execute SQL per Book.

### Validate ownership before dependent writes

The Book UPDATE is scoped by both Book id and the transaction's user id. Its
affected-row count will be compared with the number of distinct input Books.
Any mismatch returns the same kind of not-found domain failure as a
single-entity update before relationship or event writes proceed. The enclosing
transaction ensures callers can roll back all earlier statements if any later
statement fails.

### Build event-author inputs from returned event identifiers

The event INSERT returns `(event_id, book_id)`. Rust maps each final
relationship to its Book's event id, then inserts all event-author rows with
one UNNEST statement. This explicitly snapshots the supplied aggregate state
and avoids depending on later reads of live relationship rows.

### Preserve existing database uniqueness

The existing `(user_id, book_id, author_id)` primary key remains authoritative.
The bulk relationship INSERT uses conflict-ignore behavior so a destination
relationship already present during a merge is a normal condition.

## Risks / Trade-offs

- [Parallel arrays can become misaligned] → Build every field array from the
  same ordered Book slice and cover multi-Book, distinct-value cases in tests.
- [Duplicate Book ids would make affected-row validation ambiguous] → Treat
  input cardinality as distinct aggregate identity and reject an affected-row
  mismatch; normal use cases provide unique Books from repository reads.
- [Later bulk statements can fail after live rows change] → Keep every
  statement on the supplied transaction so rollback restores atomicity.
- [Bulk SQL is more complex than single-row SQL] → Mirror `create_all()` and
  assert live relationships and event snapshots in repository tests.
