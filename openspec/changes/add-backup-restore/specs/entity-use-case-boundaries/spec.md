## MODIFIED Requirements

### Requirement: Mutation invariants are preserved
Entity Command Interactors SHALL preserve all existing transaction, repository,
lock-order, and event-recording behavior while consolidating operation-specific
Interactors, and all authenticated mutating transactions MUST acquire the shared
per-user transaction lock before entity-specific locks or writes.

#### Scenario: A mutation succeeds
- **WHEN** an existing user, book, or author mutation completes successfully
- **THEN** it performs the same repository changes and event recording as before
- **AND** transactional mutations commit at the same boundary as before

#### Scenario: A transactional mutation fails
- **WHEN** repository, event recording, or commit processing fails
- **THEN** the mutation returns the same class of error as before
- **AND** it does not commit after a pre-commit failure

#### Scenario: Authors are merged
- **WHEN** the merge-author command executes
- **THEN** source and destination validation and Book-before-Author lock order are preserved
- **AND** source deletion, destination merge-event recording, event-set ID, and transaction boundary are preserved

#### Scenario: Mutation overlaps restore for one user
- **WHEN** a normal mutation and a state or full restore target the same authenticated user concurrently
- **THEN** both transactions acquire the same stable user lock before other locks or writes
- **AND** they execute serially without interleaving

#### Scenario: Different users mutate concurrently
- **WHEN** mutating transactions target different authenticated users
- **THEN** their user locks differ and do not serialize those transactions
