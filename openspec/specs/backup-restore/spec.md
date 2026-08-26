## Purpose

Define portable, versioned current and full backup export and restore behavior,
including authentication ownership, validation, atomicity, event history, and
REST transport constraints.

## Requirements

### Requirement: Versioned current backup export
The system SHALL expose authenticated `GET /backup/current` and return a
`bookshelf-current-backup` version 1 JSON document containing the authenticated
user's Authors and Books from one consistent snapshot.

#### Scenario: Current backup is exported
- **WHEN** an authenticated user requests `GET /backup/current`
- **THEN** the response contains `format`, `version`, `exportedAt`, Authors, and Books using camelCase fields
- **AND** each Book contains its `authorIds`
- **AND** ownership fields, lookup-table rows, and join timestamps are absent

#### Scenario: Concurrent mutation occurs during export
- **WHEN** current backup selects overlap a concurrent committed mutation
- **THEN** every exported current-data collection represents the same database snapshot

### Requirement: Versioned full backup export
The system SHALL expose authenticated `GET /backup/full` and return a
`bookshelf-full-backup` version 1 JSON document containing current data and the
authenticated user's complete supported event history from one consistent snapshot.

#### Scenario: Full backup is exported
- **WHEN** an authenticated user requests `GET /backup/full`
- **THEN** the response contains current data plus event sets, Book events, and Author events
- **AND** Book event author relations are represented as `authorIds`
- **AND** each event has a backup-local `eventId` suitable for internal references

#### Scenario: Concurrent mutation occurs during full export
- **WHEN** full backup selects overlap a concurrent committed mutation
- **THEN** current data and all history collections represent the same database snapshot

### Requirement: Backup ownership follows authentication
The system MUST derive export and restore ownership exclusively from the
authenticated `Claims.sub` and MUST NOT accept or require a user ID in a backup document.

#### Scenario: User restores a portable backup
- **WHEN** user B restores a valid document exported by user A
- **THEN** the document is restored only into user B's data and history
- **AND** user A and every other user's rows remain unchanged

### Requirement: Backup version dispatch and validation
The system SHALL validate the complete backup before destructive work and SHALL
explicitly reject unsupported formats, unsupported versions, malformed values,
duplicate IDs, and invalid references as client input errors.

#### Scenario: Current backup is valid
- **WHEN** a V1 current document has unique Book and Author IDs, valid fields and enums, and every Book author reference exists
- **THEN** validation produces a current restore model

#### Scenario: Backup header is unsupported
- **WHEN** `format` or `version` does not identify a supported backup type
- **THEN** restore fails with a distinct 4xx validation response
- **AND** existing data is unchanged

#### Scenario: Current reference is invalid
- **WHEN** a Book refers to an Author absent from current backup Authors
- **THEN** restore fails with an invalid-reference response before mutation

#### Scenario: Full history is invalid
- **WHEN** event IDs are duplicated, an event set is absent, an event relation is invalid, an operation or extra schema is unsupported, or a restore source event is absent
- **THEN** full restore fails with a specific 4xx validation response before mutation

### Requirement: Atomic current restore
The system SHALL expose authenticated `POST /backup/current/restore` and replace
the authenticated user's current Authors, Books, and relations in one
transaction while preserving existing history.

#### Scenario: Current backup is restored
- **WHEN** a valid current V1 backup is posted
- **THEN** current Authors and Books are completely replaced with the supplied IDs, fields, and entity timestamps
- **AND** Book-Author relations are reconstructed with new relation timestamps
- **AND** pre-existing event history is retained

#### Scenario: Current restore records boundary snapshots
- **WHEN** a valid current restore commits
- **THEN** one pre-restore and one post-restore `snapshot_all` event set are appended
- **AND** their snapshot events have extras with version 1, reason `current_backup_restore`, and phases `before` and `after` respectively

#### Scenario: Current restore fails
- **WHEN** any current replacement or snapshot write fails
- **THEN** all current data and history remain exactly as before the request

### Requirement: Atomic full restore
The system SHALL expose authenticated `POST /backup/full/restore` and replace the
authenticated user's current data and supported history in one transaction.

#### Scenario: Full backup is restored
- **WHEN** a valid full V1 backup is posted
- **THEN** current Authors, Books, relations, event sets, and entity events are completely replaced by its semantic content
- **AND** the restore adds no new event set, restore event, or snapshot

#### Scenario: Full restore fails
- **WHEN** any deletion, insertion, mapping, or commit operation fails
- **THEN** current data and history remain exactly as before the request

### Requirement: Full restore remaps global event IDs
The system MUST treat backup event IDs as document-local references, allocate new
database event IDs, and rewrite every supported event-ID reference consistently.

#### Scenario: Full backup is restored into a populated database
- **WHEN** backup event IDs overlap database-global IDs owned by any user
- **THEN** newly inserted Book and Author events receive collision-free database IDs
- **AND** Book-event Author relations and restore `source_event_id` extras refer to the mapped IDs
- **AND** Book, Author, and event-set UUIDs remain unchanged

### Requirement: Restore payload limits
The system SHALL limit current restore request bodies to 10 MiB and full restore
request bodies to 100 MiB.

#### Scenario: Restore body exceeds its route limit
- **WHEN** a restore request body exceeds the configured limit
- **THEN** the server responds with HTTP 413 Payload Too Large
- **AND** existing data and history are unchanged

### Requirement: REST-only backup surface
The system SHALL implement backup and restore through JSON REST routes and MUST
NOT add backup fields or backup document types to the GraphQL schema.

#### Scenario: Backup routes are introduced
- **WHEN** the API schema and router are inspected
- **THEN** the four backup REST routes are present
- **AND** the GraphQL schema has no backup operations or document types
