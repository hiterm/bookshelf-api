BEGIN;

INSERT INTO bookshelf_user (id) VALUES ('user1');

INSERT INTO author
(id, user_id, name)
VALUES
('9d548ede-f636-4890-8418-ad3b7336d8e3', 'user1', 'author1');

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
  'user1',
  'title1',
  'isbn1',
  false,
  false,
  50,
  'eBook',
  'Kindle'
);

ROLLBACK;
