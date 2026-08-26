ALTER TABLE book_event
  DROP CONSTRAINT book_event_event_set_id_fkey;

ALTER TABLE author_event
  DROP CONSTRAINT author_event_event_set_id_fkey;

ALTER TABLE event_set
  DROP CONSTRAINT event_set_pkey,
  ADD PRIMARY KEY (id, user_id);

ALTER TABLE book_event
  ADD FOREIGN KEY (event_set_id, user_id)
  REFERENCES event_set (id, user_id);

ALTER TABLE author_event
  ADD FOREIGN KEY (event_set_id, user_id)
  REFERENCES event_set (id, user_id);
