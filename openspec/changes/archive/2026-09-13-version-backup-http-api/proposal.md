## Why

Backup and future explicitly versioned non-GraphQL HTTP APIs need a version
namespace, and backup responses should remain ordinary JSON APIs rather than
owning browser filenames.

## What Changes

- **BREAKING** move backup endpoints to `/v1/backup/snapshot` and `/v1/backup/full`.
- Remove the legacy endpoints, attachment header, filename generation, and CORS expose-header dependency.
- Keep the backup JSON schema unchanged and distinguish HTTP API versioning from backup format versioning.

## Capabilities

### New Capabilities

- `versioned-http-api`: Defines `/v1` for backup and future explicitly versioned non-GraphQL HTTP APIs without moving `/me` or `/health`.

### Modified Capabilities

- `backup-export`: Makes backup export an authenticated JSON API without filename semantics.

## Impact

Routing, backup HTTP responses, CORS configuration, E2E tests, and frontend consumers change.
