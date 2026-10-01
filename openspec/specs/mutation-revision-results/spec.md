# mutation-revision-results Specification

## Purpose
Define explicit per-entity revision results across repository and use-case boundaries so mutation response metadata identifies the revision recorded for the returned entity.
## Requirements
### Requirement: Single-entity writes return explicit revision results

Book and Author repository create/update operations SHALL return a validated `RevisionNumber` identifying the revision recorded for that entity. Restore SHALL return the restored entity together with its newly recorded revision number. Use cases MUST construct response revision metadata directly from these results after a successful commit, preserving the public GraphQL schema.

#### Scenario: Create and update propagate independent results

- **WHEN** a Book or Author create/update repository returns a revision number other than 1
- **THEN** the mutation response contains that exact revision number

#### Scenario: Restore identifies the newly appended revision

- **WHEN** an owned historical revision is restored
- **THEN** the response number equals the new persisted revision and its OperationChange after revision, rather than the requested source revision

#### Scenario: Failed mutation has no success payload

- **WHEN** repository writes or transaction commit fail
- **THEN** the use case returns an error without a success payload

### Requirement: Transaction context excludes entity revision results

Transactions SHALL retain operation and user context without storing a mutable last-entity revision result. Bulk writes SHALL retain their aggregate result contracts without exposing a last revision.

#### Scenario: Multiple entities have different revision sequences

- **WHEN** multiple entity revisions are appended in a transaction
- **THEN** each explicit result identifies its own entity revision independently of subsequent writes

