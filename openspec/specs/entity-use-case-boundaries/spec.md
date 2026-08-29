## Purpose

Define entity-scoped Query and Command UseCase boundaries that reduce application
layer coupling while preserving static dispatch, mutation invariants, and the
externally observable GraphQL behavior.
## Requirements
### Requirement: Entity-scoped application boundaries
The application layer SHALL expose User, Book, and Author operations through separate Query and Command UseCase boundaries per entity, and SHALL expose Operation and Revision reads through history Query UseCase boundaries.

#### Scenario: Query dependencies are entity scoped
- **WHEN** a presentation component reads users, Books, Authors, Operations, or Revisions
- **THEN** it depends directly on the corresponding entity or history Query UseCase and not an aggregate cross-entity facade

#### Scenario: Command dependencies are entity scoped
- **WHEN** a presentation component mutates users, Books, or Authors
- **THEN** it depends directly on the corresponding entity Command UseCase and not an operation-specific or aggregate mutation facade

### Requirement: Static dispatch is preserved
The entity-scoped and history UseCases and Interactors SHALL use generic static dispatch and MUST NOT require dynamic trait objects.

#### Scenario: Application dependencies are composed
- **WHEN** the application constructs GraphQL roots and loaders
- **THEN** concrete generic Interactor types satisfy their UseCase bounds at compile time and no `dyn` UseCase boundary is introduced

### Requirement: Mutation invariants are preserved
Entity Command Interactors SHALL preserve transaction, repository, lock-order, and atomic Revision-recording behavior while using one Operation for each logical command.

#### Scenario: A mutation succeeds
- **WHEN** a user, Book, or Author mutation completes successfully
- **THEN** current state, Revisions, and OperationChanges commit at the same use-case-owned boundary

#### Scenario: A transactional mutation fails
- **WHEN** repository, Revision recording, OperationChange recording, or commit processing fails
- **THEN** the mutation returns the corresponding error class and commits no partial state

#### Scenario: Authors are merged
- **WHEN** the merge-Author command executes
- **THEN** source and destination validation and Book-before-Author lock order are preserved and all affected entity changes share one Operation

### Requirement: External GraphQL behavior follows the history migration
The application SHALL preserve unrelated entity fields, inputs, validation, and batch-loading behavior while intentionally replacing Event/EventSet history, restore arguments, and mutation metadata with Operation/Revision contracts.

#### Scenario: Existing entity operations execute
- **WHEN** the updated E2E suite exercises user registration, Book and Author CRUD, import, restore, merge, undo, and history queries
- **THEN** non-history entity behavior remains compatible and history behavior matches the new Operation/Revision schema

#### Scenario: Batch queries execute
- **WHEN** Book-by-Author, Author-by-ID, or OperationChange DataLoader batches resolve
- **THEN** the corresponding Query UseCase returns the expected grouped result without N+1 repository calls

### Requirement: Regression assets track the final schema
The implementation SHALL migrate unit, integration, E2E, fixture, generated-schema, and frontend contract assets to the Operation/Revision model.

#### Scenario: Regression verification completes
- **WHEN** implementation verification is performed
- **THEN** formatting, lint, unit, database integration, migration, E2E, schema, and OpenSpec checks pass without legacy Event contracts
