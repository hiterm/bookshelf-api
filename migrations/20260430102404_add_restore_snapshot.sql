-- Rename history_operation to event_operation
ALTER TABLE history_operation RENAME TO event_operation;

-- Add restore and snapshot operation types
INSERT INTO event_operation VALUES ('restore'), ('snapshot');

-- Add restore_book, restore_author, snapshot to event_set_operation
INSERT INTO event_set_operation VALUES
  ('restore_book'), ('restore_author'), ('snapshot');

-- Add extra column to event tables (operation-specific additional data)
ALTER TABLE book_event ADD COLUMN extra jsonb;
ALTER TABLE author_event ADD COLUMN extra jsonb;

-- Insert snapshot events for all existing entities (one event_set per user)
WITH user_ids AS (
  SELECT DISTINCT user_id FROM book
  UNION
  SELECT DISTINCT user_id FROM author
),
new_sets AS (
  INSERT INTO event_set (id, user_id, operation)
  SELECT gen_random_uuid(), user_id, 'snapshot'
  FROM user_ids
  RETURNING id, user_id
),
new_book_events AS (
  INSERT INTO book_event
    (event_set_id, operation, book_id, user_id,
     title, isbn, read, owned, priority, format, store,
     book_created_at, book_updated_at)
  SELECT
    ns.id, 'snapshot', b.id, b.user_id,
    b.title, b.isbn, b.read, b.owned, b.priority, b.format, b.store,
    b.created_at, b.updated_at
  FROM book b
  JOIN new_sets ns ON b.user_id = ns.user_id
  RETURNING event_id, book_id
),
_book_event_authors AS (
  INSERT INTO book_event_author (event_id, author_id)
  SELECT nbe.event_id, ba.author_id
  FROM new_book_events nbe
  JOIN book_author ba ON ba.book_id = nbe.book_id
)
INSERT INTO author_event
  (event_set_id, operation, author_id, user_id,
   name, yomi, author_created_at, author_updated_at)
SELECT
  ns.id, 'snapshot', a.id, a.user_id,
  a.name, a.yomi, a.created_at, a.updated_at
FROM author a
JOIN new_sets ns ON a.user_id = ns.user_id;
