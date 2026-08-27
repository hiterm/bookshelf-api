CREATE TABLE operation (
  id                   uuid        NOT NULL PRIMARY KEY,
  user_id              text        NOT NULL REFERENCES bookshelf_user(id),
  type                 text        NOT NULL CHECK (type IN (
    'baseline',
    'create_book',
    'update_book',
    'delete_book',
    'restore_book',
    'create_author',
    'update_author',
    'delete_author',
    'restore_author',
    'import_books',
    'merge_author',
    'restore_backup',
    'undo'
  )),
  detail               jsonb,
  undo_of_operation_id uuid,
  created_at           timestamptz NOT NULL DEFAULT current_timestamp,
  CHECK (detail IS NULL OR jsonb_typeof(detail) = 'object'),
  CHECK (undo_of_operation_id IS NULL OR undo_of_operation_id <> id),
  UNIQUE (id, user_id),
  FOREIGN KEY (undo_of_operation_id, user_id)
    REFERENCES operation(id, user_id)
);

CREATE INDEX operation_user_created_at_idx
  ON operation (user_id, created_at DESC, id DESC)
  WHERE type <> 'baseline';

CREATE INDEX operation_undo_of_operation_id_idx
  ON operation (undo_of_operation_id)
  WHERE undo_of_operation_id IS NOT NULL;

CREATE TABLE book_revision (
  book_id             uuid        NOT NULL,
  revision_number     integer     NOT NULL CHECK (revision_number > 0),
  user_id             text        NOT NULL REFERENCES bookshelf_user(id),
  title               text        NOT NULL,
  isbn                text        NOT NULL,
  read                boolean     NOT NULL,
  owned               boolean     NOT NULL,
  priority            integer     NOT NULL,
  format              text        NOT NULL REFERENCES book_format(format) ON UPDATE CASCADE,
  store               text        NOT NULL REFERENCES book_store(store) ON UPDATE CASCADE,
  book_created_at     timestamptz NOT NULL,
  book_updated_at     timestamptz NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (user_id, book_id, revision_number)
);

CREATE INDEX book_revision_user_book_number_idx
  ON book_revision (user_id, book_id, revision_number DESC);

CREATE TABLE book_revision_author (
  user_id         text    NOT NULL REFERENCES bookshelf_user(id),
  book_id         uuid    NOT NULL,
  revision_number integer NOT NULL,
  author_id       uuid    NOT NULL,
  PRIMARY KEY (user_id, book_id, revision_number, author_id),
  FOREIGN KEY (user_id, book_id, revision_number)
    REFERENCES book_revision(user_id, book_id, revision_number) ON DELETE CASCADE
);

CREATE INDEX book_revision_author_author_id_idx
  ON book_revision_author (author_id);

CREATE TABLE author_revision (
  author_id           uuid        NOT NULL,
  revision_number     integer     NOT NULL CHECK (revision_number > 0),
  user_id             text        NOT NULL REFERENCES bookshelf_user(id),
  name                text        NOT NULL,
  yomi                text        NOT NULL,
  author_created_at   timestamptz NOT NULL,
  author_updated_at   timestamptz NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT current_timestamp,
  PRIMARY KEY (user_id, author_id, revision_number)
);

CREATE INDEX author_revision_user_author_number_idx
  ON author_revision (user_id, author_id, revision_number DESC);

CREATE TABLE book_operation_change (
  operation_id          uuid    NOT NULL,
  user_id               text    NOT NULL,
  book_id               uuid    NOT NULL,
  before_revision_number integer,
  after_revision_number  integer,
  PRIMARY KEY (operation_id, book_id),
  CHECK (before_revision_number IS NOT NULL OR after_revision_number IS NOT NULL),
  FOREIGN KEY (operation_id, user_id)
    REFERENCES operation(id, user_id) ON DELETE CASCADE,
  FOREIGN KEY (user_id, book_id, before_revision_number)
    REFERENCES book_revision(user_id, book_id, revision_number),
  FOREIGN KEY (user_id, book_id, after_revision_number)
    REFERENCES book_revision(user_id, book_id, revision_number)
);

CREATE INDEX book_operation_change_book_id_idx
  ON book_operation_change (book_id, operation_id);

CREATE TABLE author_operation_change (
  operation_id          uuid    NOT NULL,
  user_id               text    NOT NULL,
  author_id             uuid    NOT NULL,
  before_revision_number integer,
  after_revision_number  integer,
  PRIMARY KEY (operation_id, author_id),
  CHECK (before_revision_number IS NOT NULL OR after_revision_number IS NOT NULL),
  FOREIGN KEY (operation_id, user_id)
    REFERENCES operation(id, user_id) ON DELETE CASCADE,
  FOREIGN KEY (user_id, author_id, before_revision_number)
    REFERENCES author_revision(user_id, author_id, revision_number),
  FOREIGN KEY (user_id, author_id, after_revision_number)
    REFERENCES author_revision(user_id, author_id, revision_number)
);

CREATE INDEX author_operation_change_author_id_idx
  ON author_operation_change (author_id, operation_id);

WITH affected_users AS (
  SELECT user_id FROM book
  UNION
  SELECT user_id FROM author
),
baseline_operations AS (
  INSERT INTO operation (id, user_id, type)
  SELECT gen_random_uuid(), user_id, 'baseline'
  FROM affected_users
  RETURNING id, user_id
),
baseline_book_revisions AS (
  INSERT INTO book_revision (
    book_id,
    revision_number,
    user_id,
    title,
    isbn,
    read,
    owned,
    priority,
    format,
    store,
    book_created_at,
    book_updated_at
  )
  SELECT
    b.id,
    1,
    b.user_id,
    b.title,
    b.isbn,
    b.read,
    b.owned,
    b.priority,
    b.format,
    b.store,
    b.created_at,
    b.updated_at
  FROM book b
  RETURNING book_id, revision_number, user_id
),
baseline_book_revision_authors AS (
  INSERT INTO book_revision_author (user_id, book_id, revision_number, author_id)
  SELECT bbr.user_id, bbr.book_id, bbr.revision_number, ba.author_id
  FROM baseline_book_revisions bbr
  JOIN book_author ba
    ON ba.book_id = bbr.book_id
   AND ba.user_id = bbr.user_id
),
baseline_author_revisions AS (
  INSERT INTO author_revision (
    author_id,
    revision_number,
    user_id,
    name,
    yomi,
    author_created_at,
    author_updated_at
  )
  SELECT
    a.id,
    1,
    a.user_id,
    a.name,
    a.yomi,
    a.created_at,
    a.updated_at
  FROM author a
  RETURNING author_id, revision_number, user_id
),
baseline_book_changes AS (
  INSERT INTO book_operation_change (
    operation_id,
    user_id,
    book_id,
    before_revision_number,
    after_revision_number
  )
  SELECT bo.id, bbr.user_id, bbr.book_id, NULL, bbr.revision_number
  FROM baseline_book_revisions bbr
  JOIN baseline_operations bo ON bo.user_id = bbr.user_id
),
baseline_author_changes AS (
  INSERT INTO author_operation_change (
    operation_id,
    user_id,
    author_id,
    before_revision_number,
    after_revision_number
  )
  SELECT bo.id, bar.user_id, bar.author_id, NULL, bar.revision_number
  FROM baseline_author_revisions bar
  JOIN baseline_operations bo ON bo.user_id = bar.user_id
  RETURNING operation_id
)
SELECT COUNT(*) FROM baseline_author_changes;
