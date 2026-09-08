## MODIFIED Requirements

### Requirement: Operations are tenant scoped and queryable
The system SHALL expose owned Operation list and single-item queries, SHALL exclude internal baseline Operations from the normal list, and SHALL resolve selected Book and Author changes and their before and after Revisions through user-scoped batch loading whose Revision retrieval query count does not grow with the number of changes.

#### Scenario: Scalar-only operation query
- **WHEN** a client selects only Operation scalar fields
- **THEN** the system does not query Book or Author changes

#### Scenario: Multiple operations select changes
- **WHEN** a client selects Book or Author changes for multiple Operations
- **THEN** each selected change kind is loaded in one user-scoped batch and changes expose their before and after Revisions

#### Scenario: Operation with many changes selects Revisions
- **WHEN** a client selects before and after Revisions for an Operation containing many Book and Author changes
- **THEN** the system retrieves Book Revisions in user-scoped batches and Author Revisions in user-scoped batches without issuing one Revision query per selected change field

#### Scenario: Change has no Revision on one side
- **WHEN** a selected Operation change has a null before or after Revision number
- **THEN** the corresponding GraphQL Revision field is null without requesting a Revision lookup for that side

#### Scenario: Requested Revision does not exist
- **WHEN** an Operation change refers to a Revision key that is not visible to the authenticated user
- **THEN** the corresponding GraphQL Revision field preserves the existing missing-Revision result and does not expose another user's Revision

#### Scenario: Operation belongs to another user
- **WHEN** a client requests an Operation owned by another user
- **THEN** the system does not return that Operation or any of its changes
