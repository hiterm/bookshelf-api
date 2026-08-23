## Why

Operation-specific UseCase traits and Interactors make every new command propagate
additional generic parameters through presentation, dependency injection, and test
builders, while the existing query facade groups unrelated entities too broadly.
Reorganizing these boundaries by entity and Query/Command responsibility reduces
that type-level coupling without changing the public GraphQL contract.

## What Changes

- Replace operation-specific UseCase traits and Interactors with entity-scoped
  Query and Command abstractions for User, Book, and Author.
- Consolidate event and event-set reads behind an Event Query abstraction.
- Remove the aggregate Query and Mutation UseCase/Interactor facades.
- Make GraphQL roots and loaders depend directly on the entity-scoped UseCases.
- Preserve static dispatch, transaction boundaries, event recording behavior,
  batch-query return shapes, and all GraphQL fields, inputs, outputs, and payloads.
- Migrate unit and presentation tests to the new responsibility boundaries while
  leaving all E2E tests, fixtures, and expectations unchanged.

## Capabilities

### New Capabilities

- `entity-use-case-boundaries`: Defines the entity-by-Query/Command application
  boundaries and the requirement that the refactor preserve external behavior.

### Modified Capabilities

None. Existing externally observable requirements do not change.

## Impact

- Affects UseCase traits and Interactors, presentation Query/Mutation roots,
  DataLoaders, dependency injection aliases, mocks, and unit tests.
- Removes the current QueryUseCase, QueryInteractor, MutationUseCase,
  MutationInteractor, and operation-specific UseCase/Interactor types.
- Does not change repository contracts, GraphQL schema, API behavior, dependencies,
  or E2E test assets.
