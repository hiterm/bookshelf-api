# backup-export Specification

## Purpose
TBD - created by archiving change add-backup-export. Update Purpose after archive.
## Requirements
### Requirement: Authenticated users can download versioned backups
The system SHALL expose authenticated `GET /backup/snapshot` and
`GET /backup/full` endpoints returning an `application/json` attachment with
format `bookshelf-backup`, version `1`, requested scope, one RFC 3339 UTC
`exportedAt`, and filename
`bookshelf-backup-{scope}-YYYY-MM-DDTHHMMSSZ.json` from the same instant.

#### Scenario: Download either scope
- **WHEN** an authenticated user requests a backup endpoint
- **THEN** the response downloads the corresponding version 1 JSON backup

#### Scenario: Reject an unauthenticated request
- **WHEN** authentication is absent or invalid
- **THEN** the existing authentication mechanism rejects it without backup data

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

### Requirement: Full backups are consistent and referentially valid
The system SHALL read all collections through one read-only repeatable database
snapshot and SHALL emit only unique, internally resolvable entity, Operation,
Revision, change, author, and undo identities as applicable.

#### Scenario: Writes commit during export
- **WHEN** concurrent writes occur
- **THEN** every exported collection reflects the same database snapshot

#### Scenario: A reference cannot be resolved
- **WHEN** exported history contains a dangling change or undo reference
- **THEN** the request fails instead of downloading an invalid backup

### Requirement: Backup arrays have deterministic order
The system SHALL order entities by ID, Operations by `createdAt` then ID,
Revisions by entity ID then revision number, and changes by entity ID.

#### Scenario: Export unchanged data repeatedly
- **WHEN** unchanged data is exported again
- **THEN** all arrays retain the same ordering

