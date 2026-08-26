## ADDED Requirements

### Requirement: Versioned snapshot backup export
The system SHALL expose authenticated `GET /backup/snapshot` and return a
`bookshelf-snapshot-backup` version 1 JSON document containing the authenticated
user's Authors, Books, and Book-Author relations from one consistent snapshot.

#### Scenario: Snapshot backup is exported
- **WHEN** an authenticated user requests `GET /backup/snapshot`
- **THEN** the response contains `format`, `version`, `exportedAt`, Authors, and Books using camelCase fields
- **AND** each Book contains its `authorIds`
- **AND** ownership fields and relation timestamps are absent

#### Scenario: Concurrent mutation occurs during export
- **WHEN** snapshot backup reads overlap a concurrent committed mutation
- **THEN** every exported current-data collection represents the same database snapshot

### Requirement: Versioned full backup export
The system SHALL expose authenticated `GET /backup/full` and return a
`bookshelf-full-backup` version 1 JSON document containing current data and the
authenticated user's supported event history from one consistent snapshot.

#### Scenario: Full backup contains history
- **WHEN** an authenticated user requests `GET /backup/full`
- **THEN** the response contains current data, event sets, Book events, and Author events
- **AND** Book-event Author relations are represented as `bookEvents[].authorIds`
- **AND** no row owned by another user is included

#### Scenario: Full backup is retained without restore
- **WHEN** a client retains a full backup
- **THEN** it can be used for history retention, incident investigation, audit, or future migration planning
- **AND** the API provides no operation that imports or restores the full document

### Requirement: Backup ownership follows authentication
The system MUST derive export, snapshot-validation, and snapshot-restore ownership exclusively from
authenticated `Claims.sub` and MUST NOT accept a user ID in a backup document.

#### Scenario: User restores a portable snapshot backup
- **WHEN** user B restores a valid snapshot document exported by user A
- **THEN** only user B's current data is replaced
- **AND** every other user's rows remain unchanged

### Requirement: Snapshot backup validation
The system SHALL expose authenticated `POST /backup/snapshot/validate`, dispatch
snapshot backup format/version, and validate the complete document including
required fields, UUIDs, timestamps, known values, duplicate Book/Author IDs, and
Book Author references without writing to the database.

#### Scenario: Valid snapshot is checked before restore
- **WHEN** an authenticated user posts a valid document to `POST /backup/snapshot/validate`
- **THEN** validation succeeds and reports Book and Author counts
- **AND** current data and event history remain unchanged

#### Scenario: Snapshot backup is invalid
- **WHEN** format, version, structure, value, duplicate, or reference validation fails
- **THEN** snapshot validate returns an invalid result and snapshot restore returns the same validation result as a stable 4xx response
- **AND** current data and history remain unchanged

#### Scenario: Validate and restore share a validator
- **WHEN** the same snapshot document is submitted to validate and restore
- **THEN** both operations use the same validation implementation
- **AND** restore revalidates before every attempt and performs no destructive work after validation failure

### Requirement: Atomic snapshot restore
The system SHALL expose authenticated `POST /backup/snapshot/restore` and replace
the authenticated user's current Authors, Books, and relations in one
transaction while retaining all event history.

#### Scenario: Snapshot backup is restored
- **WHEN** a valid snapshot V1 backup is posted
- **THEN** current Authors and Books are completely replaced with supplied IDs, fields, and entity timestamps
- **AND** Book-Author relations are reconstructed
- **AND** pre-existing event history is retained

#### Scenario: Restore records boundary snapshots
- **WHEN** a snapshot restore commits
- **THEN** pre-restore and post-restore `snapshot_all` event sets are appended
- **AND** their events have version 1 extras with reason `snapshot_backup_restore` and phases `before` and `after`

#### Scenario: Restore fails
- **WHEN** validation, replacement, snapshot recording, or commit fails
- **THEN** current data and history remain exactly as before the request

### Requirement: Snapshot restore serialization
Snapshot restore and normal mutations MUST share a stable transaction-scoped
per-user lock acquired before entity-specific locks or writes.

#### Scenario: Mutation overlaps snapshot restore
- **WHEN** a normal mutation and snapshot restore target the same user concurrently
- **THEN** they execute serially without interleaving

### Requirement: Snapshot input payload limit
The system SHALL limit snapshot validate and restore request bodies to 10 MiB.

#### Scenario: Restore body exceeds the limit
- **WHEN** a snapshot validate or restore request exceeds 10 MiB
- **THEN** the server responds with HTTP 413 Payload Too Large
- **AND** current data and history remain unchanged

### Requirement: Full restore is not exposed
The system MUST NOT expose full validate or restore routes, handlers, use cases,
event-history write paths, or request limits because full backup is read-only output.

#### Scenario: Backup API surface is inspected
- **WHEN** the router and GraphQL schema are inspected
- **THEN** the REST surface contains only `GET /backup/snapshot`, `POST /backup/snapshot/validate`, `POST /backup/snapshot/restore`, and `GET /backup/full`
- **AND** the GraphQL schema has no backup operations or document types
