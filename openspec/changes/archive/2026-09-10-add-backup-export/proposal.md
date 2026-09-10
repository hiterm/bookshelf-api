## Why

Users cannot create a portable copy of their library or retained history. A
versioned application-level export is needed before restore can be designed.

## What Changes

- Add authenticated JSON download endpoints for snapshot and full backups.
- Export current Books and Authors without authentication-provider identity.
- Export complete retained Operation and Revision history in full backups.
- Read every component from one consistent database snapshot.
- Return deterministic JSON with a timestamped download filename.

## Capabilities

### New Capabilities

- `backup-export`: Export versioned snapshot and full library backups.

### Modified Capabilities

None.

## Impact

Adds a backup read model, PostgreSQL queries, use case, HTTP routes, DTOs, and
tests. It adds no migration and does not change GraphQL.
