## Context

The event history API uses separate use-case and GraphQL models for list and detail access to the same domain `EventSet`. The detail use case eagerly loads book and author events, while the list model omits them. The schema should represent the domain concept consistently without turning a list query into an unconditional nested-event load. This change spans use cases, repositories, GraphQL request context, generated schema, tests, and the separately maintained `bookshelf` client schema.

## Goals / Non-Goals

**Goals:**

- Represent event sets with one GraphQL `EventSet` type and one scalar-only use-case DTO.
- Resolve nested event fields lazily and batch loads across all event sets in a request.
- Preserve user scoping, response fields, field nullability, query field names, and UI behavior.
- Coordinate the schema-breaking type rename with regenerated frontend types.

**Non-Goals:**

- Changing the domain `EventSet` or event-recording model.
- Adding a new GraphQL query or user-facing event-history workflow.
- Renaming the `EventSetDetail` UI component, whose name describes screen responsibility.
- Embedding nested events in `EventSetDto`.

## Decisions

### Keep `EventSetDto` scalar-only

Both list and single-item use cases return `EventSetDto` containing `id`, `operation`, and `created_at`. `find_event_set` stops coordinating nested repositories. This prevents the query resolver from eagerly loading data that the GraphQL selection does not require. Keeping a detail DTO with optional or deferred fields was rejected because it retains two representations or leaks presentation loading concerns into the use-case model.

### Add user-scoped batch repository and use-case operations

Book and author event repositories accept multiple event-set IDs and execute one set-based query constrained by `user_id`. Use cases convert the resulting domain events to event DTOs and return flat vectors. Presentation code groups them by event-set ID. Returning a map from repositories or naming APIs around maps was rejected because grouping is a DataLoader concern.

The existing single-ID repository operations remain where other callers need them. Empty ID inputs return an empty vector without issuing an invalid query.

### Resolve nested fields through two DataLoaders

`BookEventsByEventSetLoader` and `AuthorEventsByEventSetLoader` use the existing async-graphql DataLoader pattern and request-scoped user context. Each loader calls its corresponding batch use case once per collected key set and returns a value for every requested event-set ID, including empty vectors. A single loader returning both event kinds was rejected because GraphQL selections must independently avoid loading unrequested fields.

### Expose one complex GraphQL object

One `EventSet` simple object contains scalar fields and uses complex-object resolvers for `bookEvents` and `authorEvents`. Both `eventSets` and `eventSet` return this type and neither query resolver loads nested events. The removed GraphQL names make this a schema-level breaking change even though field response shapes remain unchanged.

### Update the frontend by schema regeneration

The API SDL is copied to the frontend's local schema and `pnpm run generate` regenerates operation types. Existing operations already select the required list and detail fields, so component behavior and component names remain unchanged unless generated-type compilation exposes a necessary minimal adjustment.

## Risks / Trade-offs

- [Direct clients depend on removed type names] → Treat the rename as breaking, document it in the API PR, and update the paired frontend in a coordinated PR.
- [DataLoader grouping omits keys with no rows] → Initialize requested keys with empty vectors and test zero-event event sets.
- [Batch SQL leaks cross-user events] → Retain an explicit `user_id` predicate and add repository coverage for mixed ownership.
- [A resolver bypasses DataLoader and reintroduces N+1] → Keep nested fields only on complex resolvers and verify batch use-case call counts where the test seam permits.
- [Separate repositories cannot deploy atomically] → Merge/deploy the API schema and frontend regeneration in coordination; rollback each repository to its prior schema/generated code if needed.

## Migration Plan

1. Add and test batch repository/use-case operations and DataLoaders.
2. Replace the API DTO and GraphQL type split, then regenerate and validate SDL.
3. Run API unit, repository, GraphQL, and existing event-history regression suites.
4. Update the frontend local SDL, regenerate types, and run unit, type, and relevant E2E checks.
5. Publish linked API and frontend PRs that call out the breaking type rename.

## Open Questions

None. The existing repository and DataLoader conventions determine concrete SQL and dependency-injection details during implementation.
