CREATE TABLE bookshelf_user (
  id text NOT NULL PRIMARY KEY,
  created_at timestamp with time zone NOT NULL default current_timestamp,
  updated_at timestamp with time zone NOT NULL default current_timestamp
);

CREATE TABLE book_format (
  format text NOT NULL PRIMARY KEY
);

INSERT INTO book_format VALUES
  ('eBook'),
  ('Printed'),
  ('Unknown');

CREATE TABLE book_store (
  store text NOT NULL PRIMARY KEY
);

INSERT INTO book_store VALUES
  ('Kindle'),
  ('Unknown');

CREATE TABLE book (
  id uuid NOT NULL,
  user_id text NOT NULL,
  title text NOT NULL,
  isbn text NOT NULL,
  read boolean NOT NULL,
  owned boolean NOT NULL,
  priority integer NOT NULL,
  format text NOT NULL,
  store text NOT NULL,
  created_at timestamp with time zone NOT NULL default current_timestamp,
  updated_at timestamp with time zone NOT NULL default current_timestamp,
  PRIMARY KEY (id, user_id),
  FOREIGN KEY (user_id) REFERENCES bookshelf_user(id),
  FOREIGN KEY (format) REFERENCES book_format(format) ON UPDATE CASCADE,
  FOREIGN KEY (store) REFERENCES book_store(store) ON UPDATE CASCADE
);

CREATE TABLE author (
  id uuid NOT NULL,
  user_id text NOT NULL,
  name text NOT NULL,
  yomi text NOT NULL default '',
  created_at timestamp with time zone NOT NULL default current_timestamp,
  updated_at timestamp with time zone NOT NULL default current_timestamp,
  PRIMARY KEY (id, user_id),
  FOREIGN KEY (user_id) REFERENCES bookshelf_user(id)
);

CREATE TABLE book_author (
  user_id text NOT NULL,
  book_id uuid NOT NULL,
  author_id uuid NOT NULL,
  created_at timestamp with time zone NOT NULL default current_timestamp,
  updated_at timestamp with time zone NOT NULL default current_timestamp,
  PRIMARY KEY (user_id, book_id, author_id),
  FOREIGN KEY (user_id) REFERENCES bookshelf_user(id),
  FOREIGN KEY (book_id, user_id) REFERENCES book(id, user_id),
  FOREIGN KEY (author_id, user_id) REFERENCES author(id, user_id)
);
