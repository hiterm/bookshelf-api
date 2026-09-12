# backup-export Specification

## Purpose
Provide authenticated users with a portable, versioned JSON export of their
current Books and Authors and, for full backups, all retained Operation and
Revision history. This capability covers export only; importing, restoring,
server-side storage, encryption, compression, and scheduling are outside its
scope.
## Requirements
### Requirement: Authenticated users can download versioned backups
The system SHALL expose authenticated `GET /v1/backup/snapshot` and
`GET /v1/backup/full` endpoints returning `application/json` with
`Cache-Control: no-store`, format `bookshelf-backup`, backup file format
version `1`, requested scope, and one RFC 3339 UTC `exportedAt`. The response
SHALL NOT prescribe a download filename through `Content-Disposition`. The
`/v1` route prefix is the HTTP API version and is independent of the body
format version.

#### Scenario: Download either scope
- **WHEN** an authenticated user requests a versioned backup endpoint
- **THEN** the response contains the corresponding version 1 JSON backup without attachment filename metadata

#### Scenario: Reject an unauthenticated request
- **WHEN** authentication is absent or invalid
- **THEN** the existing authentication mechanism rejects it without backup data

#### Scenario: Reject a legacy route
- **WHEN** a client requests `/backup/snapshot` or `/backup/full`
- **THEN** no backup route is available

### Requirement: Snapshot losslessly represents current state
Snapshot data SHALL contain every current owned Author and Book with stable IDs,
all persistent and nullable fields, lifecycle timestamps, date-only
`purchaseDate`, and Book `authorIds`, and SHALL contain no history data.

#### Scenario: Export a populated or empty library
- **WHEN** a user exports a snapshot
- **THEN** all and only current owned values and relations appear in stable arrays

### Requirement: Full backup losslessly represents retained history
Full data SHALL include the complete snapshot plus every retained Operation,
including baseline, detail and undo links, every Book and Author Revision, and
all before/after changes nested under their Operation.

#### Scenario: Export complete history
- **WHEN** history contains create, update, delete, detail, undo, and revision
  Author relations
- **THEN** all values, nullable links, and revision `authorIds` are preserved

### Requirement: Backups are portable and tenant isolated
The system SHALL export only records owned by the authenticated user and SHALL
not serialize `user_id` or any authentication-provider identity. Backup export
SHALL remain a use-case-specific query concern until import or restore defines
a recoverable state as a domain concept. Its infrastructure implementation
SHALL implement only a use-case-owned backup query port and SHALL NOT cause the
domain layer to depend on use-case types.

#### Scenario: Another tenant owns data
- **WHEN** another user owns entities or history
- **THEN** none of those records or identity appears in the backup

#### Scenario: The backup query is wired to PostgreSQL
- **WHEN** the backup interactor obtains its consistent export projection
- **THEN** it depends on a use-case-owned query port implemented by
  infrastructure without introducing a domain-to-use-case dependency

### Requirement: Full backups use a consistent database snapshot
The system SHALL read current state, Operations, Revisions, changes, and
relations through one read-only repeatable database snapshot so every exported
collection represents the same persisted instant. Backup export SHALL serialize
the tenant-owned persisted records visible in that snapshot without independently
revalidating that every stored identity or reference resolves.

#### Scenario: Writes commit during export
- **WHEN** concurrent writes occur
- **THEN** every exported collection reflects the same database snapshot

### Requirement: Backup arrays have deterministic order
The system SHALL order entities by ID, Operations by `createdAt` then ID,
Revisions by entity ID then revision number, and changes by entity ID.

#### Scenario: Export unchanged data repeatedly
- **WHEN** unchanged data is exported again
- **THEN** all arrays retain the same ordering
