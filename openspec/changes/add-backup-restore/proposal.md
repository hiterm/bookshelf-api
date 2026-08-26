## Why

Users need a portable, versioned way to preserve and recover both their current
bookshelf and its event history without exposing database-specific rows. The API
currently has no atomic backup or restore boundary, so recovery and migration
between environments cannot be performed safely.

## What Changes

- Add authenticated REST endpoints to export and restore current and full
  backups as versioned JSON documents.
- Define stable V1 current and full backup contracts that omit ownership data
  and flatten book-author event relations into author ID arrays.
- Validate complete backup documents before mutation, enforce per-route request
  size limits, and return client-facing errors for invalid input.
- Restore current data atomically while retaining history and recording before
  and after `snapshot_all` event sets.
- Restore current data and event history atomically for full backups, remapping
  database-global event IDs and all versioned references without recording a
  restore event.
- Serialize all mutations for the same authenticated user with a shared
  transaction-scoped locking protocol while allowing different users to proceed
  independently.
- Add unit and REST E2E coverage and document the event-recording exceptions and
  transaction policy.

## Capabilities

### New Capabilities

- `backup-restore`: Versioned current/full backup JSON, authenticated REST
  export and restore behavior, validation, atomicity, history semantics, event
  ID remapping, request limits, and user isolation.

### Modified Capabilities

- `entity-use-case-boundaries`: Mutating transactions must participate in a
  shared per-user locking protocol so normal mutations and restores cannot
  interleave.

## Impact

- Adds four REST routes and backup-specific presentation, use-case, domain, and
  PostgreSQL infrastructure boundaries; the GraphQL schema remains unchanged.
- Changes transaction startup for all existing mutations by acquiring a stable
  PostgreSQL advisory lock for the authenticated user.
- Adds no public dependency on database schema versions and does not accept or
  restore a backup-provided user ID.
- Adds unit and database-backed E2E tests and updates event-recording
  architecture documentation.
