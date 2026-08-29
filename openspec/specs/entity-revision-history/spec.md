# entity-revision-history Specification

## Purpose
TBD - created by archiving change redesign-history-model. Update Purpose after archive.
## Requirements
### Requirement: Entity revisions are complete append-only snapshots
The system SHALL store complete Book and Author snapshots as append-only Revisions identified within an authenticated tenant by entity ID and a positive revision number, while the ordinary entity tables remain authoritative current state. Database keys MUST include the owner so equal entity UUIDs belonging to different users do not collide.

#### Scenario: Book revision is recorded
- **WHEN** a Book current state is created or changed
- **THEN** its Revision contains all scalar fields, lifecycle timestamps, owner, audit time, and every aggregate Author ID

#### Scenario: Author revision is recorded
- **WHEN** an Author current state is created or changed
- **THEN** its Revision contains all scalar fields, lifecycle timestamps, owner, and audit time

#### Scenario: Historical revision exists
- **WHEN** a later mutation changes the same entity
- **THEN** the system appends a new Revision and does not update or delete the earlier Revision

### Requirement: Revision numbers are monotonic per entity
The system MUST assign revision numbers starting at 1 and increasing by one independently for each Book and Author.

#### Scenario: Entity is changed repeatedly
- **WHEN** an entity whose latest revision is N is updated or restored
- **THEN** the new snapshot is revision N+1

#### Scenario: Entity is deleted
- **WHEN** an entity at revision N is deleted
- **THEN** no deleted-state Revision is created and revision N remains its latest complete snapshot

### Requirement: Current entities receive baseline revisions
The schema migration SHALL create revision 1 and a baseline OperationChange for every Book and Author that exists at migration time without converting legacy Event history.

#### Scenario: Existing Book has Authors
- **WHEN** the migration snapshots an existing Book
- **THEN** revision 1 contains its current scalars and current Book-Author references under its owner's baseline Operation

#### Scenario: Existing Author is present
- **WHEN** the migration snapshots an existing Author
- **THEN** revision 1 contains its current state under its owner's baseline Operation

#### Scenario: Legacy history exists
- **WHEN** legacy Event rows predate the migration
- **THEN** the migration does not copy them into Revisions or OperationChanges

### Requirement: Revisions are tenant scoped and queryable
The system SHALL expose list and exact lookup queries for Book and Author Revisions using entity ID and revision number and MUST enforce ownership on every lookup.

#### Scenario: Client lists Book revisions
- **WHEN** an authenticated owner requests `bookRevisions(bookId)`
- **THEN** the system returns that Book's complete Revisions in a deterministic revision order

#### Scenario: Client requests exact Author revision
- **WHEN** an authenticated owner requests `authorRevision(authorId, revisionNumber)`
- **THEN** the system returns only that exact owned snapshot

#### Scenario: Revision belongs to another user
- **WHEN** a client requests another user's entity revision
- **THEN** the system does not return it
