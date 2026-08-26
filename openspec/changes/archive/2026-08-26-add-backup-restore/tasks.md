## 1. Export Contracts

- [x] 1.1 Add camelCase SnapshotBackupV1 and FullBackupV1 envelopes and serialization contract tests
- [x] 1.2 Export snapshot Authors, Books, and flattened Book-Author relations from one consistent snapshot
- [x] 1.3 Export full event sets, Book events, Author events, and flattened Book-event Author relations from the same snapshot
- [x] 1.4 Ensure exports derive ownership only from Claims.sub and exclude other users

## 2. Snapshot Validation and Restore

- [x] 2.1 Implement one SnapshotBackupV1 validator for format/version, required fields, UUIDs, timestamps, known values, duplicate IDs, and Author references
- [x] 2.2 Add authenticated read-only POST /backup/snapshot/validate with summary and validation errors
- [x] 2.3 Reuse the identical validator in snapshot restore before any destructive repository call
- [x] 2.4 Implement atomic snapshot restore with retained history and before/after snapshot_all events using versioned snapshot_backup_restore extras
- [x] 2.5 Apply the shared per-user transaction lock to snapshot restore and normal mutations
- [x] 2.6 Apply the 10 MiB limit to snapshot validate and restore

## 3. Deliberately Omitted Full Input APIs

- [x] 3.1 Remove the full-restore route, handler, use case, history write path, event-ID mapping, reference rewriting, and related tests
- [x] 3.2 Do not add full/validate because full backup is read-only output with no current input workflow
- [x] 3.3 Remove the event-set schema migration that was needed only for cross-user full restore
- [x] 3.4 Record the safety rationale, secondary implementation complexity, and separate-future-change policy in proposal and design

## 4. Tests and Documentation

- [x] 4.1 Add unit tests for valid snapshot validation, invalid format/version, duplicate IDs, missing Author references, and validator reuse
- [x] 4.2 Add REST E2E for read-only validation, matching validate/restore rejection, authentication, request limits, snapshots, full export contents, relations, and isolation
- [x] 4.3 Document snapshot naming, snapshot_all conceptual consistency, current-restore event semantics, and GET-only full backup positioning
- [x] 4.4 Confirm the router exposes only the four intended REST endpoints and GraphQL remains unchanged

## 5. Verification and Delivery

- [x] 5.1 Run cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, and cargo test --locked
- [x] 5.2 Run database-backed backup E2E and confirm the GraphQL schema has no backup diff
- [x] 5.3 Synchronize delta specs, run strict OpenSpec validation, and archive the change
