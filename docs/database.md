# Database Design

## Current state

`book`, `author`, and `book_author` are the authoritative current state. Every
row is scoped by `user_id`; relationship keys include the same owner to prevent
cross-tenant references.

## Operation and Revision history

Each logical mutation creates one owned `operation`. Its `type`, typed JSON
`detail`, optional `undo_of_operation_id`, and database-managed `created_at`
describe the mutation without embedding entity snapshots. Baseline Operations
seed history for entities that existed when the Revision schema was introduced
and are hidden from normal Operation lists.

`book_revision` and `author_revision` contain immutable full snapshots. Their
primary keys are `(user_id, entity_id, revision_number)`, and revision numbers
start at 1 and increase independently for every owned entity. Book Author
membership is snapshotted in `book_revision_author` using the same owner and
revision identity.

`book_operation_change` and `author_operation_change` connect an Operation to
each affected entity. Nullable before and after revision numbers represent:

- create: `NULL -> revision 1`
- update or restore: `revision N -> revision N+1`
- delete: `revision N -> NULL`

At least one side must be present. Composite foreign keys for Operation owner
and Revision owner prevent cross-tenant history links.

## Atomicity and undo

The use-case layer creates the Operation, mutates current state, appends all
Revisions, and inserts all OperationChanges in one PostgreSQL transaction.
Import preview executes the same writes and explicitly rolls the transaction
back.

Undo creates another ordinary `operation` with `type = 'undo'` and records its
target in `undo_of_operation_id`. It locks and revalidates all affected entities
before applying inverse state, then records fresh Revisions and changes in the
same transaction.

The legacy Event/EventSet tables and lookup tables are removed after the
Operation/Revision baseline migration. Existing current state and
Operation/Revision history do not depend on them.
