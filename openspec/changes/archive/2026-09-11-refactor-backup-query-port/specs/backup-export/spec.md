## MODIFIED Requirements

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
