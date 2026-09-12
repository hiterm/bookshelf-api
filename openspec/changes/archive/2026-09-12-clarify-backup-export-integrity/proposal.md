## Why

Backup export currently promises to revalidate persisted references, which expands a read-only export into a database integrity checker without defining the necessary validation and error contract. The requirement should instead match the implemented responsibility: faithfully export tenant-owned current state and retained history from one consistent database snapshot.

## What Changes

- Remove the requirement that backup export fail when a persisted history reference cannot be resolved.
- Remove the requirement that the export layer independently prove every exported identity or reference is internally resolvable.
- Preserve tenant isolation, complete retained-history export, and the single read-only repeatable-read snapshot guarantee.
- Strengthen HTTP E2E coverage for the exact version 1 JSON schema, representative values, authentication, and timestamp consistency.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `backup-export`: Limit export-time integrity responsibility to faithfully serializing persisted current state and history from a consistent database snapshot.

## Impact

- Updates the canonical `backup-export` requirement after delta synchronization.
- Adds E2E contract assertions without changing the production endpoint or JSON format.
- Does not add database corruption detection, reference revalidation, or a new error contract.
