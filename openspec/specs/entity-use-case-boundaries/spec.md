## Purpose

Define entity-scoped Query and Command UseCase boundaries that reduce application
layer coupling while preserving static dispatch, mutation invariants, and the
externally observable GraphQL behavior.

## Requirements

### Requirement: Entity-scoped application boundaries
The application layer SHALL expose User, Book, and Author operations through
separate Query and Command UseCase boundaries per entity, and SHALL expose event
and event-set reads through an Event Query UseCase boundary.

#### Scenario: Query dependencies are entity scoped
- **WHEN** a presentation component reads users, books, authors, or events
- **THEN** it depends directly on the corresponding entity Query UseCase
- **AND** it does not depend on an aggregate cross-entity Query facade

#### Scenario: Command dependencies are entity scoped
- **WHEN** a presentation component mutates users, books, or authors
- **THEN** it depends directly on the corresponding entity Command UseCase
- **AND** it does not depend on operation-specific UseCases or an aggregate Mutation facade

### Requirement: Static dispatch is preserved
The entity-scoped UseCases and Interactors SHALL use generic static dispatch and
MUST NOT require dynamic trait objects.

#### Scenario: Application dependencies are composed
- **WHEN** the application constructs GraphQL roots and loaders
- **THEN** concrete generic Interactor types satisfy their UseCase bounds at compile time
- **AND** no `dyn` UseCase boundary is introduced

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

### Requirement: External GraphQL behavior is unchanged
The refactor SHALL NOT change the GraphQL schema or externally observable API
behavior, including field names, arguments, inputs, outputs, payloads, and batch
loading results.

#### Scenario: Existing GraphQL operations execute
- **WHEN** the unchanged E2E suite exercises user registration and query, book and author CRUD, import, restore, merge, event queries, and DataLoader fields
- **THEN** every operation produces the same schema-visible result and behavior as before

#### Scenario: Batch queries execute
- **WHEN** book-by-author or author-by-ID DataLoader batches are resolved
- **THEN** the entity Query UseCase returns the existing `HashMap`-based result shape

### Requirement: Regression assets remain unchanged
The implementation SHALL migrate unit and presentation tests to entity-scoped
boundaries while leaving existing E2E code, fixtures, and expectations unchanged.

#### Scenario: Regression verification completes
- **WHEN** implementation verification is performed
- **THEN** formatting, lint, unit, integration, and E2E checks pass
- **AND** E2E assets and the GraphQL schema have no refactor-induced diff
