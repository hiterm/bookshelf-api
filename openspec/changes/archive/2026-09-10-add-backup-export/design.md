## Context

Current Books and Authors are stored separately from authoritative Operation,
OperationChange, and Revision history. Records are tenant scoped by provider
user ID, while a portable backup must not be. The implementation starts from
backend `1ecd4a139b7956238a87dc3274b52a1e891131f6` and frontend
`67a1c979f4a377d2e8cdd7c40f2540727a026778`.

## Goals / Non-Goals

**Goals:** define `bookshelf-backup` version 1; preserve current state and, for
full scope, all retained history; guarantee tenant isolation, referential
integrity, deterministic ordering, and a transactionally consistent view.

**Non-Goals:** import, restore, migration, encryption, compression, scheduling,
server-side storage, database-layout exposure, or GraphQL changes.

## Decisions

1. Authenticated `GET /backup/snapshot` and `/backup/full` return JSON
   attachments. One UTC instant supplies `exportedAt` and filename timestamp.
2. A dedicated repository reads through one read-only repeatable-read
   transaction, including current state and all history for full scope.
3. Current and revision Books contain logical `authorIds`; changes are nested
   under Operations. No join-table arrays or `user_id` are serialized.
4. Arrays sort by ID; Operations by creation time then ID; Revisions by entity
   ID then number; changes by entity ID. Invalid references fail the export.

## Risks / Trade-offs

- **[Large histories consume memory]** -> Version 1 materializes one JSON
  response; streaming can be added later.
- **[Concurrent writes create inconsistencies]** -> Use one repeatable-read
  database snapshot.

## Migration Plan

Deploy before the frontend. No database migration is required.
