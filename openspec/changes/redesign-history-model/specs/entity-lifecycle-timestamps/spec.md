## MODIFIED Requirements

### Requirement: Restore records a new lifecycle update
The system SHALL preserve an entity's historical creation timestamp when restoring a Book or Author Revision and SHALL set its update timestamp to the restore Operation timestamp.

#### Scenario: Restore an entity snapshot
- **WHEN** a Book or Author is restored from a historical Revision
- **THEN** the restored entity retains the snapshot creation timestamp and receives the restore Operation timestamp as its update timestamp

#### Scenario: Restore a deleted entity
- **WHEN** a deleted Book or Author is restored from a Revision
- **THEN** the recreated current entity and newly appended Revision share the preserved creation time and new update time

### Requirement: Audit time remains database-managed
The system SHALL treat Operation and Revision creation timestamps as database-recording times independent of entity lifecycle timestamps.

#### Scenario: Record a mutation revision
- **WHEN** a Book or Author mutation is committed
- **THEN** PostgreSQL records Operation and Revision audit timestamps without requiring them to equal entity lifecycle timestamps

