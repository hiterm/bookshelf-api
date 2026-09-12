## RENAMED Requirements

- FROM: `### Requirement: Full backups are consistent and referentially valid`
- TO: `### Requirement: Full backups use a consistent database snapshot`

## MODIFIED Requirements

### Requirement: Full backups use a consistent database snapshot
The system SHALL read current state, Operations, Revisions, changes, and relations through one read-only repeatable database snapshot so every exported collection represents the same persisted instant. Backup export SHALL serialize the tenant-owned persisted records visible in that snapshot without independently revalidating that every stored identity or reference resolves.

#### Scenario: Writes commit during export
- **WHEN** concurrent writes occur
- **THEN** every exported collection reflects the same database snapshot
