## Context

The application layer currently has two opposing granularity problems. Commands
are modeled as operation-specific traits and Interactors, causing the aggregate
Mutation facade and its consumers to accumulate generic parameters whenever an
operation is added. Queries are collected in a single cross-entity Query facade,
which couples unrelated repositories and consumers. Presentation and DataLoader
code depend on these facades even though each field needs only one entity's
capability.

This is an internal refactor. Static dispatch, repository contracts,
`TransactionManager` transaction association, event recording, GraphQL schema,
and runtime behavior are constraints. Existing E2E assets must remain untouched.

## Goals / Non-Goals

**Goals:**

- Make entity × Query/Command the basic UseCase and Interactor boundary.
- Let GraphQL Query, Mutation, and loaders depend directly on the smallest
  applicable entity UseCase.
- Preserve mutation transaction/event invariants and every external GraphQL
  field, argument, input, output, payload, and behavior.
- Preserve static dispatch and existing test coverage at the new boundaries.

**Non-Goals:**

- Splitting repository traits into read/write variants.
- Introducing CQRS read models or dynamic trait objects.
- Renaming EventQueryUseCase to HistoryQueryUseCase.
- Changing batch-query return values, GraphQL schema, or E2E assets.
- Creating an ExecPlan.

## Decisions

### Entity-scoped UseCase traits

Create `UserQueryUseCase`/`UserCommandUseCase`,
`BookQueryUseCase`/`BookCommandUseCase`,
`AuthorQueryUseCase`/`AuthorCommandUseCase`, and `EventQueryUseCase` in entity
trait modules. Query method names omit redundant entity names:

- User query: `find_by_id`; command: `register`.
- Book query: `find_by_id`, `find_all`, `find_by_author_ids`; command: `create`,
  `update`, `delete`, `import`, `restore`.
- Author query: `find_by_id`, `find_all`, `find_by_ids`; command: `create`,
  `update`, `delete`, `merge`, `restore`.
- Event query: `list_book_events`, `list_author_events`, `list_event_sets`,
  `find_event_set`.

Entity boundaries reduce dependency propagation while retaining meaningful
cohesion. Operation-specific traits were rejected because they caused the
current generic explosion; a single application facade was rejected because it
preserves excessive coupling.

### Entity-scoped Interactors with static dispatch

Implement:

- `UserQueryInteractor<UR>` and `UserCommandInteractor<UR>`.
- `BookQueryInteractor<BR>` and `BookCommandInteractor<BR, AR, BER, TM>`.
- `AuthorQueryInteractor<AR>` and `AuthorCommandInteractor<AR, BR, AER, TM>`.
- `EventQueryInteractor<BER, AER, ESR>`.

All remain generic concrete types. `dyn trait` was rejected to preserve static
dispatch and the project's current compile-time dependency model.

### Preserve transaction type equality and mutation invariants

Command Interactors retain `TransactionManager` and bounds tying repository
associated transaction types to `TM::Transaction`. Existing implementations are
moved rather than redesigned. In particular, author merge preserves validation,
Book-before-Author lock order, source deletion, destination merge-event append,
event-set ID, transaction boundary, and no-commit-on-failure behavior. Restore,
import, repository failure, and commit failure semantics remain unchanged.

Event repositories remain with the commands that record mutations; restore is
not moved into the event query abstraction.

### Presentation depends directly on entity UseCases

GraphQL roots become `Query<UQ, BQ, AQ, EQ>` and `Mutation<UC, BC, AC>`, with
one generic per entity responsibility. They delegate directly to those UseCases,
without Query/Mutation application facades. Loaders depend on Book or Author
query UseCases and call `find_by_author_ids` or `find_by_ids`; their existing
`HashMap` return shape is retained.

Dependency injection defines entity-scoped concrete aliases where useful and
injects them directly into roots. This removes the aggregate facade types rather
than recreating them under another name.

### Tests move with responsibility, not behavior

Interactor unit tests move beside the new entity Interactors, retaining normal,
failure, transaction, commit-failure, merge, restore, and import cases.
Presentation tests use new entity UseCase mocks. Delegation-only facade tests are
removed with their facades. Existing E2E code, fixtures, and expectations are
not edited; passing them is the regression proof for the external contract.

## Risks / Trade-offs

- **Risk: Moving command logic changes transaction or event ordering.** → Move
  implementations mechanically first and retain the existing failure-focused
  tests, then inspect merge/import/restore paths explicitly.
- **Risk: Renamed query methods leak into the GraphQL contract.** → Restrict
  renames to Rust trait methods and verify the generated/schema snapshot has no
  diff plus run unchanged E2E tests.
- **Risk: Broad generic changes create difficult compiler errors.** → Migrate
  entity boundaries in dependency order: traits, Interactors/tests,
  presentation/loaders, then DI and deletion.
- **Trade-off: Each Query/Mutation root still has several generic parameters.** →
  This is accepted because each parameter is stable per entity responsibility
  and preserves static dispatch without an artificial facade.

## Migration Plan

1. Add entity-scoped traits and Interactors and migrate their tests.
2. Rewire presentation roots, loaders, mocks, and dependency injection.
3. Remove old facades and operation-specific types after all references are gone.
4. Run formatting, lint, unit/integration tests, and unchanged E2E tests; verify
   E2E files and GraphQL schema have no diff.
5. Roll back by reverting this internal refactor; no data or deployment migration
   is required.

## Open Questions

None.
