## Why

Users need portable, versioned exports of their current bookshelf and event
history. Snapshot restore is also needed to replace live Books and Authors while
retaining an auditable boundary in existing history.

## What Changes

- Add authenticated `GET /backup/snapshot` and `GET /backup/full` exports.
- Add authenticated read-only `POST /backup/snapshot/validate` and reuse its
  validator in `POST /backup/snapshot/restore`.
- Add snapshot restore with a 10 MiB
  request limit, atomic current-data replacement, and before/after snapshots.
- Serialize snapshot restore with normal mutations for the same authenticated
  user through the shared transaction-scoped lock.
- Intentionally do not expose full restore.

## Decision: Do not provide full restore

The API provides full backup but not full restore. Full export has concrete
value for retaining history, investigating incidents and defects, audit work,
and preserving material for a future migration or restore design. Export and
restore are intentionally asymmetric because full restore has no concrete use
case today.

The primary reason is safety. Full restore would be a destructive operation
capable of replacing both current data and event history. Publishing an unused
destructive API increases the chance of future accidental or incorrect calls
and resulting data loss without delivering present value. The danger of making
an operation available when it is not planned for use outweighs API symmetry.

Implementation complexity is a secondary reason, not the deciding one. The
current `BIGSERIAL` event IDs would require reallocation, backup-to-database ID
mapping, rewriting `book_event_author` and `extra.source_event_id` references,
whole-history integrity validation, and a dedicated history write path. Those
costs can be justified if a real use case appears.

Any future full restore will be designed in a separate OpenSpec change and PR,
not added retroactively here. That design must reconsider the allowed use case,
misoperation safeguards, pre-restore state, EventId UUID/UUIDv7 migration,
event-ID compatibility and references, target integrity, and cross-environment
or cross-user migration policy.

## Capabilities

### New Capabilities

- `backup-restore`: Versioned snapshot/full exports, snapshot validation, and atomic snapshot restore,
  including validation, history semantics, request limits, and user isolation.

### Modified Capabilities

- `entity-use-case-boundaries`: Mutating transactions share a stable per-user
  lock so normal mutations and snapshot restore cannot interleave.

## Impact

- Adds four authenticated REST routes; the GraphQL schema remains unchanged.
- Adds no event-history replacement path and no database migration.
- Adds unit and REST E2E coverage and documents snapshot-restore event semantics.
