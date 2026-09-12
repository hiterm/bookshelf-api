## MODIFIED Requirements

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
