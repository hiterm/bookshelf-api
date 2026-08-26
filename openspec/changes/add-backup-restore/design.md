## Context

Bookshelf API stores live Books and Authors separately from an append-only event
log. Existing mutating use cases open `PgTransaction` through
`TransactionManager::begin`, which immediately creates an event set. Backup
export needs a consistent snapshot across several tables, while restore needs a
transaction that can replace rows without accidentally producing normal entity
events. Full restore also has to translate backup-local event identifiers to the
database-global `BIGSERIAL` namespace.

The new API is REST-only and authenticated by the existing `Claims` extractor.
Backup documents never contain ownership; `claims.sub` is the sole restore
target. PostgreSQL is the only supported infrastructure implementation.

## Goals / Non-Goals

**Goals:**

- Define explicit `CurrentBackupV1` and `FullBackupV1` JSON contracts that can
  remain readable when later versions are introduced.
- Export current data and history from one consistent database snapshot.
- Validate an entire document before destructive writes and restore atomically.
- Preserve current-restore history while adding before/after snapshots, and
  replace full-restore history without adding a restore event.
- Serialize restore with every normal mutation for one user.
- Keep backup-specific SQL behind a narrow repository boundary and backup logic
  out of existing Book/Author interactors and GraphQL.

**Non-Goals:**

- Browser/file UI, multipart uploads, server-side backup storage, compression,
  streaming, or object storage.
- GraphQL backup fields or types.
- Automatically exporting a backup immediately before full restore.
- Preserving `book_author` timestamps or physical event `BIGSERIAL` values.

## Decisions

### Version envelope and parsing

Presentation receives the JSON body as a typed version-dispatch envelope. The
dispatcher first reads only `format` and `version`, rejects unknown pairs, then
deserializes the complete V1 type with camelCase fields and strict known enum
values. Separate V1 DTOs remain public within the backup boundary and convert to
validated internal restore models. This avoids tying compatibility to database
migrations or silently interpreting future formats as V1.

Current documents use `bookshelf-current-backup`; full documents use
`bookshelf-full-backup`. Both have version `1`, `exportedAt`, and `data` with
Authors and Books. Full documents additionally contain `history`. Ownership and
join-table timestamps are absent, and both live and historical book-author
relations are flattened to `authorIds`.

Alternative: serialize database rows directly. Rejected because column changes,
ownership fields, lookup tables, and physical join/event identifiers would make
the format unsafe and brittle.

### Backup-specific repository and transaction boundary

A narrow `BackupRepository` exposes current/full snapshot reads and current/full
replacement operations using backup-specific models. Its PostgreSQL adapter owns
the multi-table SQL, ordering, event-ID insertion mapping, and reference rewrite.
The backup interactor owns format dispatch and semantic validation.

Export starts a read-only `REPEATABLE READ` transaction so all selected tables
share a snapshot. Restore starts a write transaction with the shared user lock
but does not automatically create an event set. Current restore explicitly asks
the repository to append before and after `snapshot_all` event sets in that same
transaction. Full restore inserts only the supplied history.

Alternative: reuse normal entity repository mutations. Rejected because they
generate events, allocate IDs and timestamps, and cannot efficiently express an
atomic complete replacement.

### Shared per-user locking

Every mutating `PgTransactionManager::begin` acquires
`pg_advisory_xact_lock(hashtextextended(user_id, 0))` before inserting its event
set. Backup restore obtains the identical transaction-scoped advisory lock before
reading its before-state or deleting rows. PostgreSQL releases the lock at commit
or rollback. A single fixed SQL expression avoids application hash instability;
locking occurs before entity rows are locked, providing one global lock order.

Alternative: row-lock `bookshelf_user`. It could work, but advisory locking
states the protocol directly, avoids coupling to a mutable row access pattern,
and naturally scopes different user IDs independently.

### Current restore event semantics

After full validation, current restore locks the user, captures and inserts a
`snapshot_all` event set with every pre-restore Book and Author, replaces live
relations/Books/Authors while preserving supplied IDs and timestamps, then
inserts a second snapshot from the post-restore state. Snapshot event `extra`
contains `{version:1, reason:"current_backup_restore", phase:"before|after"}`.
Existing history is never deleted. Any failure rolls back snapshots and data.

### Full restore event-ID translation

Backup `eventId` values are document-local unique signed integers. After semantic
validation, full restore deletes the target user's old history and live data,
then inserts current rows, event sets, and Author/Book events. Each event insert
returns its new global database ID and records a mapping keyed by entity kind and
backup event ID. Book-event author rows use the Book mapping. Version-1 restore
`extra.source_event_id` values are rewritten through the same entity-kind map;
other supported versioned extras are validated and retained. Missing or
cross-kind references fail validation before writes.

Event set, Book, and Author UUIDs are preserved. Full restore creates no event set
of its own and no before/after snapshots.

### HTTP boundaries and limits

The router mounts four authenticated JSON routes. Current restore has a 10 MiB
`DefaultBodyLimit`; full restore has 100 MiB. Oversize payloads map to 413.
Malformed/unsupported/invalid documents map to stable 4xx JSON errors, while
transaction and database failures map according to existing presentation error
conventions. Exports are ordinary JSON responses in V1.

## Risks / Trade-offs

- [Large full exports allocate the complete document in memory] → Accept for V1,
  enforce restore limits, and defer streaming/compression to a later change.
- [Advisory locks only work when every mutation participates] → Centralize the
  normal path in `PgTransactionManager::begin` and test restore/mutation blocking
  with the same user and independence across users.
- [History extras can acquire new reference schemas] → Use explicit versioned
  extra parsers and reject unsupported schemas instead of copying unknown JSON.
- [Deletes and reinserts can violate foreign keys] → Delete join/event rows in
  dependency order and insert parent/current rows before dependents.
- [A valid but semantically surprising timeline may be imported] → Validate IDs,
  references, operations, required nullable snapshots, extras, and timestamp
  ordering defined by V1; do not attempt to reconstruct intent beyond the V1
  contract.

## Migration Plan

Deploy by applying `20260826000000_scope_event_sets_to_users.sql`, which changes
the `event_set` primary key and entity-event foreign keys to include `user_id`,
before starting application instances with the four routes and shared advisory
lock. A rollback of the application can leave the composite keys in place because
older queries remain compatible. Reversing the schema itself requires first
proving event-set UUIDs are globally unique, then restoring the single-column
primary and foreign keys; it must not be attempted after cross-account restores
have introduced duplicate UUIDs.

## Open Questions

None. Initial request limits are fixed at 10 MiB for current and 100 MiB for
full; streaming and compression are deferred.
