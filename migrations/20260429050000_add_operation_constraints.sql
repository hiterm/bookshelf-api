CREATE TABLE history_operation (
    operation text NOT NULL PRIMARY KEY
);

INSERT INTO history_operation VALUES
    ('create'),
    ('update'),
    ('delete');

CREATE TABLE change_set_operation (
    operation text NOT NULL PRIMARY KEY
);

INSERT INTO change_set_operation VALUES
    ('create_book'),
    ('update_book'),
    ('delete_book'),
    ('create_author'),
    ('update_author'),
    ('delete_author');

ALTER TABLE book_history
    ADD CONSTRAINT book_history_operation_fk
    FOREIGN KEY (operation) REFERENCES history_operation(operation);

ALTER TABLE author_history
    ADD CONSTRAINT author_history_operation_fk
    FOREIGN KEY (operation) REFERENCES history_operation(operation);

ALTER TABLE change_set
    ADD CONSTRAINT change_set_operation_fk
    FOREIGN KEY (operation) REFERENCES change_set_operation(operation);
