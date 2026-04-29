ALTER TABLE book_history
    ADD CONSTRAINT book_history_operation_check
    CHECK (operation IN ('create', 'update', 'delete'));

ALTER TABLE author_history
    ADD CONSTRAINT author_history_operation_check
    CHECK (operation IN ('create', 'update', 'delete'));

ALTER TABLE change_set
    ADD CONSTRAINT change_set_operation_check
    CHECK (operation IN (
        'create_book', 'update_book', 'delete_book',
        'create_author', 'update_author', 'delete_author'
    ));
