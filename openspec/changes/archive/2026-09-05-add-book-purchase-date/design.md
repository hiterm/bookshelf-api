## Context

Book state flows through PostgreSQL rows, the domain model, command/query DTOs,
GraphQL presentation objects, and immutable revision snapshots. Purchase date
must cross every path without acquiring time or timezone semantics.

## Goals / Non-Goals

**Goals:**

- Store an optional calendar date on current books and revisions.
- Preserve the field through create, full update, import, preview, history,
  restore, and undo workflows.
- Expose one nullable GraphQL `Date` contract throughout.

**Non-Goals:**

- Inferring or backfilling purchase dates for existing rows.
- Adding purchase-date validation beyond representing a valid calendar date.
- Introducing PATCH-style update semantics.

## Decisions

- PostgreSQL uses nullable `date` columns and Rust uses `Option<time::Date>`.
  This preserves date-only meaning without a new value object or timezone.
- `UpdateBookInput.purchaseDate: Date` maps directly to `Option<Date>` in the
  existing full-replacement update DTO. Null therefore explicitly clears the
  field; a nested optional is unnecessary and would change the contract.
- Book revisions store the value as a snapshot column. Existing generic
  restore and undo flows reconstruct the domain book from that snapshot.
- Bulk import extends the existing UNNEST arrays with a nullable date array so
  query count and atomicity remain unchanged.

## Risks / Trade-offs

- [A missed conversion path silently drops the value] → Extend tests across
  repository, import, revision, restore, undo, and GraphQL boundaries.
- [Migration rollback discards entered purchase dates] → Use reversible column
  drops and document normal backup requirements for rollback.

## Migration Plan

Apply nullable columns to `book` and `book_revision`; no table rewrite or
backfill is required. Deploy application code after migration. Rollback drops
the two columns after reverting application code.

## Open Questions

None.
