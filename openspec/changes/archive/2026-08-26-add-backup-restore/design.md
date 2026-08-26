## Context

Bookshelf API stores live Books and Authors separately from an append-only event
log. Export needs a consistent snapshot across several tables. Snapshot restore
must replace live rows atomically without deleting history and must serialize
with normal same-user mutations. The REST API uses the existing `Claims`
extractor, and `claims.sub` is always the owner and target.

## Goals / Non-Goals

**Goals:**

- Define explicit versioned `SnapshotBackupV1` and `FullBackupV1` JSON exports.
- Export each document from one consistent database snapshot.
- Validate snapshot backup completely before destructive writes.
- Replace current data atomically while retaining history and recording
  before/after `snapshot_all` boundaries.
- Preserve full history as a read-only export for retention, investigation,
  audit, and possible future migration work.

**Non-Goals:**

- Full restore or any event-history replacement/write path.
- Event-ID allocation, mapping, or reference rewriting during backup import.
- Browser/file UI, multipart uploads, server-side storage, compression,
  streaming, object storage, or GraphQL backup operations.

## Decisions

### Versioned export contracts

Snapshot documents use `bookshelf-snapshot-backup`; full documents use
`bookshelf-full-backup`. Both use version 1, camelCase fields, `exportedAt`, and
current Authors and Books. Books flatten relations into `authorIds`. Full
documents additionally contain event sets, Book events, Author events, and
Book-event Author relations flattened into `bookEvents[].authorIds`. Ownership
and join timestamps are not exported.

Exports use read-only `REPEATABLE READ` transactions so all collections share
one database snapshot. A narrow `BackupRepository` owns the multi-table reads.

### Snapshot restore

Snapshot validate and restore accept JSON with a 10 MiB body limit. One shared
validator dispatches format/version, strictly deserializes `SnapshotBackupV1`,
and validates required fields, UUIDs, timestamps, fields, enums, duplicate IDs,
and Author references. Validate returns the result without any repository call.
Restore calls the identical validator on every request and begins destructive
work only after success.

The repository begins one transaction, acquires the shared per-user advisory
lock, appends a pre-restore `snapshot_all`, replaces current Book/Author/relation
rows while preserving supplied IDs and timestamps, and appends a post-restore
snapshot. Snapshot extras are
`{version:1, reason:"snapshot_backup_restore", phase:"before|after"}`. Existing
history is retained. Any failure rolls back snapshots and current data.

### Shared per-user locking

Every normal mutating transaction and snapshot restore acquires
`pg_advisory_xact_lock(hashtextextended(user_id, 0))` before entity locks or
writes. PostgreSQL releases it at commit or rollback. Different users retain
independent locks.

### Decision: snapshot naming and validation

`current` can also mean a current version or latest backup and does not directly
identify the saved content. `snapshot` means the state captured at one point in
time. Existing event `snapshot_all` captures that concept inside event history;
backup `snapshot` captures it for portable external storage. Their mechanisms
and destinations differ, but the concept is shared, so consistent terminology
is preferred.

Snapshot restore completely replaces live state and is destructive, so a
read-only validation endpoint has a concrete preflight use. Validate and restore
share one validator to prevent disagreement about restorability.

### Decision: full backup without full restore

The asymmetric API is intentional. Full export has immediate non-destructive
uses: history retention, incident and defect investigation, audits, and saving
data for a future migration or restore facility. No concrete full-restore use
case exists today.

Safety is decisive. A full restore would replace current data and event history,
so an accidental call could cause broad data loss. Publishing an unused
destructive endpoint creates operational risk without present benefit. API
symmetry is not sufficient justification.

Complexity is supplementary: `BIGSERIAL` event IDs imply reallocation and ID
maps, `book_event_author` and versioned `extra.source_event_id` rewrites,
cross-table history validation, and a dedicated write path. Complexity would be
acceptable for a concrete need, but does not justify exposing the operation now.

A future requirement must use a separate OpenSpec and PR and reconsider allowed
uses, safeguards, pre-restore state, EventId UUID/UUIDv7, ID compatibility,
reference restoration, integrity rules, and cross-environment/cross-user scope.

### Decision: no full validation endpoint

Full backup is currently server-generated read-only output and no API accepts it
as input. A separate full validator therefore has no concrete workflow. Export
correctness is covered by unit and E2E contract tests. If full import, restore,
or saved-backup inspection becomes a real use case, full validation will be
designed with that separate change. This follows the policy of exposing only
API surface needed now.

## HTTP Surface

- `GET /backup/snapshot`
- `POST /backup/snapshot/validate` (10 MiB limit)
- `POST /backup/snapshot/restore` (10 MiB limit)
- `GET /backup/full`

All require `Claims`; unauthenticated requests return 401. Invalid snapshot
restore documents return a stable 4xx response, oversized bodies return 413,
and database/internal failures return 5xx. There is no full restore route,
handler, use case, request limit, or history write path. There is also no
`POST /backup/full/validate` because full documents have no input workflow.

## Risks / Trade-offs

- Full exports allocate the complete document in memory; streaming and
  compression are deferred.
- Advisory locking is effective only when every mutation participates, so the
  lock is centralized in normal transaction startup and reused by restore.
- A full backup cannot currently be imported. This is deliberate; retained
  exports remain useful, and any future importer receives its own safety design.

## Migration Plan

Deploy the application with the four REST routes and shared advisory lock. No
database schema migration is required. Rollback removes the routes and lock
acquisition without transforming stored data.

## Open Questions

None.
