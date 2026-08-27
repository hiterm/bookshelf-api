# Database Design

## Operation and Revision history

The authoritative history model stores one logical `operation`, immutable full
`book_revision` / `author_revision` snapshots, and entity-specific
OperationChanges. Legacy Event tables described below temporarily coexist.

`operation` has primary key `id` and unique `(id, user_id)` ownership identity.
Its `type`, typed JSON `detail`, optional `undo_of_operation_id`, and timestamp
describe the complete logical mutation. Baseline operations seed revision 1
from live state but are hidden from normal Operation lists.

`book_revision` uses primary key `(user_id, book_id, revision_number)`;
`author_revision` uses `(user_id, author_id, revision_number)`. Book Author
membership is snapshotted in `book_revision_author`, whose foreign key includes
the same owner and revision identity. Revision numbers are positive and are
allocated per `(user_id, entity_id)`.

`book_operation_change` and `author_operation_change` carry `operation_id`,
`user_id`, entity ID, and nullable before/after revision numbers. At least one
side must be present. Composite foreign keys `(operation_id, user_id)` and
`(user_id, entity_id, revision_number)` prevent cross-tenant Operation and
Revision references at the database level.

## Overview

The event log records every state change to `book` and `author` entities.
Each operation (create, update, delete, restore, snapshot_all) produces one
`event_set` row and one or more event rows in `book_event` / `author_event`.
The event tables are append-only; live entity data lives in `book` and
`author` as before.

## Tables

### `event_set_operation`

Lookup table for valid `event_set.operation` values.

| value           | description                                      |
|-----------------|--------------------------------------------------|
| `create_book`   | A book was created                               |
| `update_book`   | A book was updated                               |
| `delete_book`   | A book was deleted                               |
| `restore_book`  | A book restore was performed                     |
| `create_author` | An author was created                            |
| `update_author` | An author was updated                            |
| `delete_author` | An author was deleted                            |
| `restore_author`| An author restore was performed                  |
| `import_books`  | A bulk import of books was performed             |
| `snapshot_all`  | A point-in-time snapshot of all entities (system)|
| `merge_author`  | One author was merged into another               |

### `event_set`

Groups one or more event rows belonging to a single logical user operation.

| column       | type        | description                                        |
|--------------|-------------|---------------------------------------------------|
| `id`         | uuid PK     | Unique identifier                                  |
| `user_id`    | text FK     | Owner (references `bookshelf_user.id`)             |
| `operation`  | text FK     | Operation type (references `event_set_operation`)  |
| `created_at` | timestamptz | When the operation occurred                        |

For `snapshot_all` operations, one `event_set` is created per user, and all of
that user's books and authors are inserted as event rows under that single
`event_set`.

### `event_operation`

Lookup table for valid per-event `operation` values.

| value      | description                                                  |
|------------|--------------------------------------------------------------|
| `create`   | Entity was created; all data fields populated                |
| `update`   | Entity was updated; all data fields populated (post-state)   |
| `delete`   | Entity was deleted; only id stored, data fields are NULL     |
| `restore`  | Entity state was restored; data fields populated, `extra` set|
| `snapshot` | Point-in-time capture; all data fields populated             |
| `merge_as_destination` | Unchanged author participated as merge destination |

### `book_event`

One row per book event. Data fields are NULL for `delete` events.

| column             | type        | description                                    |
|--------------------|-------------|------------------------------------------------|
| `event_id`         | bigserial PK| Auto-incrementing event identifier             |
| `event_set_id`     | uuid FK     | References `event_set.id`                      |
| `operation`        | text FK     | References `event_operation.operation`         |
| `book_id`          | uuid        | The book this event belongs to                 |
| `user_id`          | text        | Owner                                          |
| `title`            | text        | NULL for delete events                         |
| `isbn`             | text        | NULL for delete events                         |
| `read`             | boolean     | NULL for delete events                         |
| `owned`            | boolean     | NULL for delete events                         |
| `priority`         | integer     | NULL for delete events                         |
| `format`           | text        | NULL for delete events                         |
| `store`            | text        | NULL for delete events                         |
| `book_created_at`  | timestamptz | NULL for delete events                         |
| `book_updated_at`  | timestamptz | NULL for delete events                         |
| `changed_at`       | timestamptz | When this event was recorded                   |
| `extra`            | jsonb       | Operation-specific additional data (see below) |

### `book_event_author`

Author IDs associated with a book event. No rows exist for delete events.

| column      | type   | description                                     |
|-------------|--------|-------------------------------------------------|
| `event_id`  | bigint | References `book_event.event_id` (CASCADE delete)|
| `author_id` | uuid   | Author associated with the book at event time   |

### `author_event`

One row per author event. Data fields are NULL for `delete` events.

| column              | type        | description                                    |
|---------------------|-------------|------------------------------------------------|
| `event_id`          | bigserial PK| Auto-incrementing event identifier             |
| `event_set_id`      | uuid FK     | References `event_set.id`                      |
| `operation`         | text FK     | References `event_operation.operation`         |
| `author_id`         | uuid        | The author this event belongs to               |
| `user_id`           | text        | Owner                                          |
| `name`              | text        | NULL for delete events                         |
| `yomi`              | text        | NULL for delete events                         |
| `author_created_at` | timestamptz | NULL for delete events                         |
| `author_updated_at` | timestamptz | NULL for delete events                         |
| `changed_at`        | timestamptz | When this event was recorded                   |
| `extra`             | jsonb       | Operation-specific additional data (see below) |

## `extra` Field Schema

The `extra` column holds operation-specific data that does not warrant a
dedicated column. All schemas include a `version` key to support future
evolution without breaking existing consumers.

### `restore` operation

```json
{
  "version": 1,
  "source_event_id": <i64>
}
```

| key               | type | description                                     |
|-------------------|------|-------------------------------------------------|
| `version`         | int  | Schema version, currently `1`                   |
| `source_event_id` | int  | The `event_id` that was used as the restore source |

### All other operations

`extra` is normally `NULL` for `create`, `update`, `delete`, and `snapshot`
events. A merge source uses a `delete` event with
`{"type":"merge","version":1,"destination_author_id":"<uuid>"}`. The
unchanged destination uses `merge_as_destination` with
`{"version":1,"source_author_id":"<uuid>"}`; its snapshot columns are NULL.

## Version History

| version | date       | change                                      |
|---------|------------|---------------------------------------------|
| 1       | 2026-04-30 | Initial schema for `restore` extra data     |
