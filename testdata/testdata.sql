BEGIN;

INSERT INTO bookshelf_user (id) VALUES ('testuser1');

INSERT INTO author
(id, user_id, name)
VALUES
('9d548ede-f636-4890-8418-ad3b7336d8e3', 'testuser1', 'author1');
INSERT INTO author
(id, user_id, name)
VALUES
('54bbfcd2-e937-4da5-84fc-5984fa7b5979', 'testuser1', 'author2');

INSERT INTO book (
  id,
  user_id,
  title,
  isbn,
  read,
  owned,
  priority,
  format,
  store
)
VALUES (
  'd85f0e56-f632-4d50-b057-bab5af3d0159',
  'testuser1',
  'title1',
  'isbn1',
  false,
  false,
  50,
  'eBook',
  'Kindle'
);

INSERT INTO book_author (book_id, author_id) VALUES
('d85f0e56-f632-4d50-b057-bab5af3d0159', '9d548ede-f636-4890-8418-ad3b7336d8e3'),
('d85f0e56-f632-4d50-b057-bab5af3d0159', '54bbfcd2-e937-4da5-84fc-5984fa7b5979');

COMMIT;
