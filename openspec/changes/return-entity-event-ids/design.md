## Context

Book and Author mutations record entity snapshots in per-entity event tables
inside the same `TransactionManager` transaction that changes the entity. The
event insert already produces an `event_id`, but create/update results currently
discard it and expose only the operation-wide event set ID. Restore mutations
accept a historical event ID, so clients must query history to discover the
event just created by a mutation.

This change crosses repository, use-case, GraphQL presentation, generated
schema, documentation, and test layers. The existing event recording invariant
and transaction boundary must remain intact.

## Goals / Non-Goals

**Goals:**

- Return the newly recorded entity event ID from Book and Author create/update
  mutations.
- Keep `eventSetId` as the identifier for the complete logical operation.
- Ensure the returned event ID identifies the committed Book or Author snapshot
  and can be supplied to the corresponding restore mutation.
- Preserve atomic failure behavior: neither a mutation result nor an event ID is
  returned when persistence fails.

**Non-Goals:**

- Adding a single event ID to delete, restore, import, or registration results.
- Changing restore input semantics, database schema, or event history queries.
- Returning all event IDs for operations that record multiple entity events.

## Decisions

Repository create/update operations will return the `event_id` obtained from
the existing event `INSERT ... RETURNING event_id`. This keeps the authoritative
database-generated identifier attached to the write that created it. Inferring
the ID later from event history was rejected because it adds a query and can be
ambiguous under concurrent writes.

The database adapter will convert the PostgreSQL `BIGINT` value into the domain
newtype `EventId`. Repository contracts and `EntityMutationResultDto<T>` will
carry `EventId` instead of a bare `i64`, so unrelated numeric values cannot be
substituted accidentally. GraphQL remains the external conversion boundary and
renders `EventId` through its decimal `Display` representation.

Create/update use cases will return a dedicated generic
`EntityMutationResultDto<T>` containing `value`, `event_set_id`, and `event_id`.
The existing `MutationResultDto<T>` remains for operations that do not guarantee
one newly recorded entity snapshot. Adding an optional event ID to the generic
result was rejected because it would weaken the create/update contract and make
absence handling leak into GraphQL.

GraphQL payloads will expose the numeric repository ID as a required GraphQL
`ID` by converting it to its decimal string representation. Both Book and
Author payload types are shared by create and update, and all four resolvers
will populate the field.

Repository event insertion and entity mutation will continue inside the same
`TransactionManager` callback. The event ID becomes externally visible only
after that callback succeeds; an error returns no mutation result.

End-to-end tests will compare the returned ID and event-set ID with history,
then use the ID as restore input. This validates the cross-layer contract more
directly than testing only the schema shape.

Each create/update E2E scenario is organized into mutation, history, and
restore-compatibility phases. These tests only assert that the returned event
ID is accepted by the restore API; restore state transitions remain the
responsibility of the dedicated restore E2E suite. The four Book/Author and
create/update scenarios stay explicit instead of hiding their GraphQL payloads
behind a large shared helper.

## Risks / Trade-offs

- [A resolver omits the new non-null field] → Construct payloads from the
  dedicated result type and cover all four resolvers plus generated schema.
- [The returned ID names the wrong entity event] → Assert entity type,
  history membership, and event-set equality in unit and E2E tests.
- [A transaction error leaks an uncommitted ID] → Return the result only
  after `TransactionManager` completes successfully and retain rollback tests.
- [Shared payload types imply `eventId` for out-of-scope operations] → Keep
  delete and restore on their distinct payload types; do not add the field
  there.

## Migration Plan

This is an additive GraphQL schema change. Deploy the repository/use-case and
GraphQL changes together, regenerate the checked-in schema, and allow clients to
adopt `eventId` when ready. Rollback removes the additive GraphQL field and
internal result propagation; no data migration is necessary.

## Open Questions

None.
