CREATE TABLE event_set_operation (
  operation text NOT NULL PRIMARY KEY
);

INSERT INTO event_set_operation VALUES
  ('create_book'),
  ('update_book'),
  ('delete_book'),
  ('create_author'),
  ('update_author'),
  ('delete_author'),
  ('restore_book'),
  ('restore_author'),
  ('snapshot_all');

CREATE TABLE event_set (
  id          uuid        NOT NULL PRIMARY KEY,
  user_id     text        NOT NULL REFERENCES bookshelf_user(id),
  operation   text        NOT NULL REFERENCES event_set_operation(operation),
  created_at  timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE TABLE event_operation (
  operation text NOT NULL PRIMARY KEY
);

INSERT INTO event_operation VALUES
  ('create'),
  ('update'),
  ('delete'),
  ('restore'),
  ('snapshot');

CREATE TABLE book_event (
  event_id        bigserial   NOT NULL PRIMARY KEY,
  event_set_id    uuid        NOT NULL REFERENCES event_set(id),
  operation       text        NOT NULL REFERENCES event_operation(operation),
  book_id         uuid        NOT NULL,
  user_id         text        NOT NULL,
  title           text,
  isbn            text,
  read            boolean,
  owned           boolean,
  priority        integer,
  format          text,
  store           text,
  book_created_at timestamptz,
  book_updated_at timestamptz,
  changed_at      timestamptz NOT NULL DEFAULT current_timestamp,
  extra           jsonb
);

CREATE TABLE book_event_author (
  event_id  bigint NOT NULL REFERENCES book_event(event_id) ON DELETE CASCADE,
  author_id uuid   NOT NULL,
  PRIMARY KEY (event_id, author_id)
);

CREATE TABLE author_event (
  event_id          bigserial   NOT NULL PRIMARY KEY,
  event_set_id      uuid        NOT NULL REFERENCES event_set(id),
  operation         text        NOT NULL REFERENCES event_operation(operation),
  author_id         uuid        NOT NULL,
  user_id           text        NOT NULL,
  name              text,
  yomi              text,
  author_created_at timestamptz,
  author_updated_at timestamptz,
  changed_at        timestamptz NOT NULL DEFAULT current_timestamp,
  extra             jsonb
);

CREATE INDEX ON book_event (user_id, book_id, changed_at DESC);
CREATE INDEX ON author_event (user_id, author_id, changed_at DESC);
CREATE INDEX ON book_event (event_set_id);
CREATE INDEX ON author_event (event_set_id);

-- Baseline snapshot of all existing data (one event_set per user)
WITH user_ids AS (
  SELECT DISTINCT user_id FROM book
  UNION
  SELECT DISTINCT user_id FROM author
),
new_sets AS (
  INSERT INTO event_set (id, user_id, operation)
  SELECT gen_random_uuid(), user_id, 'snapshot_all'
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
