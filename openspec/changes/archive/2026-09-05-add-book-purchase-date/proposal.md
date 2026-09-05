## Why

Books currently record when they were added to the application, but not the
calendar date on which they were purchased. A first-class optional purchase
date is needed across normal mutations, imports, and revision workflows.

## What Changes

- Add a nullable, date-only purchase date to books and book revisions.
- Accept and return the purchase date through create, update, import, preview,
  query, history, restore, and undo GraphQL workflows.
- Preserve explicit-null full-update semantics so updating a book with a null
  purchase date clears the stored value.
- Include purchase dates in bulk persistence and revision snapshots.

## Capabilities

### New Capabilities

- `book-purchase-date`: Optional date-only book metadata across persistence,
  domain, and GraphQL APIs.

### Modified Capabilities

- `bulk-book-import`: Imports persist purchase dates for each book.
- `book-import-preview`: Import previews return normalized purchase dates.
- `entity-revision-history`: Book revision snapshots expose purchase dates.
- `revision-restore`: Restoring a book revision restores its purchase date.
- `operation-undo`: Undoing a book mutation restores its purchase date.

## Impact

Database migrations and repository SQL, book domain models and DTOs, command
and query interactors, revision/event restoration, GraphQL inputs and objects,
generated `schema.graphql`, and migration/unit/integration/E2E fixtures and
tests are affected. No new dependency or breaking API behavior is introduced.
