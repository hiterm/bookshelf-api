CREATE TABLE event_set_operation (
  operation text NOT NULL PRIMARY KEY
);

INSERT INTO event_set_operation VALUES
  ('create_book'),
  ('update_book'),
  ('delete_book'),
  ('create_author'),
  ('update_author'),
  ('delete_author');

CREATE TABLE event_set (
  id          uuid        NOT NULL PRIMARY KEY,
  user_id     text        NOT NULL REFERENCES bookshelf_user(id),
  operation   text        NOT NULL REFERENCES event_set_operation(operation),
  created_at  timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE TABLE history_operation (
  operation text NOT NULL PRIMARY KEY
);

INSERT INTO history_operation VALUES
  ('create'),
  ('update'),
  ('delete');

CREATE TABLE book_event (
  event_id        bigserial   NOT NULL PRIMARY KEY,
  event_set_id    uuid        NOT NULL REFERENCES event_set(id),
  operation       text        NOT NULL REFERENCES history_operation(operation),
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
  changed_at      timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE TABLE book_event_author (
  event_id  bigint NOT NULL REFERENCES book_event(event_id) ON DELETE CASCADE,
  author_id uuid   NOT NULL,
  PRIMARY KEY (event_id, author_id)
);

CREATE TABLE author_event (
  event_id          bigserial   NOT NULL PRIMARY KEY,
  event_set_id      uuid        NOT NULL REFERENCES event_set(id),
  operation         text        NOT NULL REFERENCES history_operation(operation),
  author_id         uuid        NOT NULL,
  user_id           text        NOT NULL,
  name              text,
  yomi              text,
  author_created_at timestamptz,
  author_updated_at timestamptz,
  changed_at        timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE INDEX ON book_event (user_id, book_id, changed_at DESC);
CREATE INDEX ON author_event (user_id, author_id, changed_at DESC);
CREATE INDEX ON book_event (event_set_id);
CREATE INDEX ON author_event (event_set_id);
