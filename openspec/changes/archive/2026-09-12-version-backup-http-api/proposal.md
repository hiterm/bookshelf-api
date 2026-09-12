## Why

Non-GraphQL HTTP APIs need an explicit version namespace, and backup responses
should remain ordinary JSON APIs rather than owning browser filenames.

## What Changes

- **BREAKING** move backup endpoints to `/v1/backup/snapshot` and `/v1/backup/full`.
- Remove the legacy endpoints, attachment header, filename generation, and CORS expose-header dependency.
- Keep the backup JSON schema unchanged and distinguish HTTP API versioning from backup format versioning.

## Capabilities

### New Capabilities

- `versioned-http-api`: Defines `/v1` as the namespace for non-GraphQL HTTP APIs.

### Modified Capabilities

- `backup-export`: Makes backup export an authenticated JSON API without filename semantics.

## Impact

Routing, backup HTTP responses, CORS configuration, E2E tests, and frontend consumers change.
