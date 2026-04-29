CREATE TABLE change_set (
  id          uuid        NOT NULL PRIMARY KEY,
  user_id     text        NOT NULL REFERENCES bookshelf_user(id),
  operation   text        NOT NULL,
  created_at  timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE TABLE book_history (
  history_id      bigserial   NOT NULL PRIMARY KEY,
  change_set_id   uuid        NOT NULL REFERENCES change_set(id),
  operation       text        NOT NULL,
  book_id         uuid        NOT NULL,
  user_id         text        NOT NULL,
  title           text        NOT NULL,
  isbn            text        NOT NULL,
  read            boolean     NOT NULL,
  owned           boolean     NOT NULL,
  priority        integer     NOT NULL,
  format          text        NOT NULL,
  store           text        NOT NULL,
  book_created_at timestamptz NOT NULL,
  book_updated_at timestamptz NOT NULL,
  changed_at      timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE TABLE book_history_author (
  history_id  bigint NOT NULL REFERENCES book_history(history_id) ON DELETE CASCADE,
  author_id   uuid   NOT NULL,
  PRIMARY KEY (history_id, author_id)
);

CREATE TABLE author_history (
  history_id        bigserial   NOT NULL PRIMARY KEY,
  change_set_id     uuid        NOT NULL REFERENCES change_set(id),
  operation         text        NOT NULL,
  author_id         uuid        NOT NULL,
  user_id           text        NOT NULL,
  name              text        NOT NULL,
  yomi              text        NOT NULL,
  author_created_at timestamptz NOT NULL,
  author_updated_at timestamptz NOT NULL,
  changed_at        timestamptz NOT NULL DEFAULT current_timestamp
);

CREATE INDEX ON book_history (user_id, book_id, changed_at DESC);
CREATE INDEX ON author_history (user_id, author_id, changed_at DESC);
CREATE INDEX ON book_history (change_set_id);
CREATE INDEX ON author_history (change_set_id);
