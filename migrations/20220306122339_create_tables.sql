CREATE TABLE bookshelf_user (
  id text NOT NULL PRIMARY KEY,
  sub text NOT NULL
);

CREATE TABLE book_format (
  format text PRIMARY KEY
);

INSERT INTO book_format VALUES
  ('eBook'),
  ('Printed');

CREATE TABLE book_store (
  store text PRIMARY KEY
);

INSERT INTO book_store VALUES
  ('Kindle');

CREATE TABLE book (
  id text NOT NULL PRIMARY KEY,
  user_id text NOT NULL,
  title text NOT NULL,
  isbn text,
  read boolean NOT NULL,
  owned boolean NOT NULL,
  priority integer NOT NULL,
  format text,
  store text,
  created_at timestamp NOT NULL,
  updated_at timestamp NOT NULL,
  FOREIGN KEY (user_id) REFERENCES bookshelf_user(id) ON UPDATE CASCADE,
  FOREIGN KEY (format) REFERENCES book_format(format) ON UPDATE CASCADE,
  FOREIGN KEY (store) REFERENCES book_store(store) ON UPDATE CASCADE
);

CREATE TABLE author (
  id text PRIMARY KEY,
  name text
);

CREATE TABLE book_author (
  id text PRIMARY KEY,
  book_id text,
  author_id text
);
