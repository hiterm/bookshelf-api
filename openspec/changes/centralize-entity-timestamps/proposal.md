## Why

Entity lifecycle timestamps are currently generated in different layers: Author creation reads the clock inside the domain entity, most Book and Author mutations read it in interactors, and imported authors rely on database defaults. This makes timestamp meaning inconsistent and prevents one logical mutation from reliably using one lifecycle time.

## What Changes

- Make the use-case layer choose one UTC lifecycle timestamp for each create, update, restore, or import operation.
- Require domain creation and update operations to receive their timestamp instead of reading the system clock.
- Preserve historical creation time during restore while setting update time to the restore operation time.
- Pass the import operation time into repository-level author lookup-or-create so newly imported entities share the operation timestamp.
- Keep event audit timestamps database-managed and separate from entity lifecycle timestamps.

## Capabilities

### New Capabilities

- `entity-lifecycle-timestamps`: Defines timestamp ownership and lifecycle behavior for Book and Author mutations.

### Modified Capabilities

None.

## Impact

The Author domain constructors, Book and Author mutation interactors, author repository interface and PostgreSQL implementation, dependency call sites, and related tests are affected. GraphQL fields, timestamp serialization, database schema, and event audit timestamp behavior remain unchanged.
