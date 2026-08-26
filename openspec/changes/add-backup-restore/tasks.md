## 1. Backup Contract and Validation

- [x] 1.1 Add explicit camelCase StateBackupV1 and FullBackupV1 DTOs, version envelopes, history/event DTOs, and serialization contract tests
- [x] 1.2 Implement format/version dispatch and state backup semantic validation with duplicate, value, and author-reference tests
- [x] 1.3 Implement full history validation, versioned event-extra parsing, and event-reference tests

## 2. Transaction and Persistence Boundaries

- [x] 2.1 Add the shared stable per-user transaction-scoped advisory lock to normal mutating transactions with database tests
- [x] 2.2 Add a narrow BackupRepository boundary and PostgreSQL read transactions that export state and full data from a consistent snapshot
- [x] 2.3 Implement atomic state replacement with preserved IDs/timestamps, retained history, and before/after snapshot_all events
- [x] 2.4 Implement atomic full replacement with history deletion/rebuild, generated event-ID mappings, and rewritten event references
- [x] 2.5 Add database tests for restore rollback, user isolation, event mapping, and same-user/different-user lock behavior

## 3. Use Cases and HTTP API

- [x] 3.1 Add backup export/state restore/full restore use-case traits and interactors using Claims-derived UserId ownership
- [x] 3.2 Add REST handlers and JSON error mapping for malformed, unsupported, invalid, conflict, and internal failures
- [x] 3.3 Wire GET/POST state/full routes into dependency injection and the main router without changing GraphQL
- [x] 3.4 Enforce 10 MiB state and 100 MiB full restore body limits and return HTTP 413 for oversized payloads
- [x] 3.5 Add handler/use-case unit tests for authentication ownership, errors, limits, snapshots, replacement semantics, and rollback

## 4. End-to-End Coverage and Documentation

- [x] 4.1 Add state backup round-trip E2E coverage including retained history and before/after snapshots
- [x] 4.2 Add full backup semantic round-trip E2E coverage including event-ID remapping and no restore event
- [x] 4.3 Add E2E coverage for cross-user isolation, invalid-backup atomicity, validation errors, and payload limits
- [x] 4.4 Document state/full restore event semantics and shared transaction locking in event-recording architecture documentation

## 5. Verification and Delivery

- [x] 5.1 Run cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, and cargo test --locked successfully
- [x] 5.2 Run the database-backed REST E2E suite successfully and confirm the GraphQL schema has no backup diff
- [x] 5.3 Validate OpenSpec artifacts and synchronize delta specs before archive
