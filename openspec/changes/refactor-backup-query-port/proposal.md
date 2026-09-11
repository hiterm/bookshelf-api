## Why

The backup query abstraction currently lives under `domain` while accepting and
returning use-case DTOs. This creates an outward `domain -> use_case`
dependency, contrary to the API's layered architecture, and leaves the
infrastructure-to-use-case dependency broader than intended.

## What Changes

- Define the backup query contract as a use-case-owned port because backup is
  currently an export-specific query projection, not a domain concept.
- Replace `domain::repository::BackupRepository` with a clearly named
  `BackupQueryPort` implemented by PostgreSQL infrastructure.
- Restrict infrastructure dependencies on use-case code to explicit
  use-case-owned ports for dedicated I/O or query projections.
- Keep external backup DTO construction in the interactor and keep the port's
  projection contract colocated with the port.
- Document the narrow exception without moving use-case-specific types into
  `domain` merely to satisfy dependency direction.

## Capabilities

### Modified Capabilities

- `backup-export`: Preserve the existing export behavior while correcting the
  ownership and dependency direction of its query abstraction.

## Impact

Refactors backup modules, dependency injection, mocks, and focused tests. HTTP
routes, version 1 JSON, database queries, transaction isolation, and frontend
contracts remain unchanged. No database migration is required.
