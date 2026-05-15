ALTER TABLE author
  ADD CONSTRAINT author_user_id_name_unique UNIQUE (user_id, name);
