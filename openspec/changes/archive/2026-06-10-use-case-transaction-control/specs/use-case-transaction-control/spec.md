## ADDED Requirements

### Requirement: Use-case layer controls transaction boundaries for Book and Author operations
The use-case layer SHALL be able to start a database transaction, invoke `BookRepository` and/or `AuthorRepository` operations within that transaction, and commit or roll back the transaction.

#### Scenario: Successful multi-repository commit
- **WHEN** a use-case interactor begins a transaction and calls `author_repository.create(&mut tx, ...)` followed by `book_repository.create(&mut tx, ...)`
- **THEN** both operations are persisted atomically when `tx.commit()` is called

#### Scenario: Transaction rollback on failure
- **WHEN** a use-case interactor begins a transaction, the first repository call succeeds, and the second repository call returns an error
- **THEN** the transaction is implicitly rolled back and no database changes from either call are persisted

### Requirement: Book and Author repository mutation methods accept an external database connection
`BookRepository` and `AuthorRepository` mutation methods (`create`, `update`, `delete`, `restore`) SHALL accept `&mut PgConnection` as their first parameter and execute all SQL against that connection.

#### Scenario: Book repository create uses injected connection
- **WHEN** `book_repository.create(&mut tx, user_id, book)` is invoked
- **THEN** the book INSERT statement executes on the provided `tx` connection, not on a newly opened connection from an internal pool

#### Scenario: Author repository update uses injected connection
- **WHEN** `author_repository.update(&mut tx, user_id, author)` is invoked
- **THEN** the UPDATE statement and associated event recording execute on the provided `tx` connection

#### Scenario: Book repository find uses injected connection inside a transaction
- **WHEN** `book_repository.find_by_id(&mut tx, user_id, book_id)` is invoked inside a use-case transaction
- **THEN** the SELECT statement executes on the provided `tx` connection, ensuring read consistency within the transaction

### Requirement: Event recording in Book and Author repositories remains atomic within the use-case transaction
Event recording (`event_set` and `*_event` table inserts) in `PgBookRepository` and `PgAuthorRepository` SHALL execute inside the same `&mut PgConnection` passed to the repository method.

#### Scenario: Create book records event in same transaction
- **WHEN** a use-case interactor calls `book_repository.create(&mut tx, user_id, book)` inside a transaction
- **THEN** both the `book` INSERT and the `event_set` + `book_event` INSERTs are committed together when the use-case calls `tx.commit()`

#### Scenario: Update author records event in same transaction
- **WHEN** a use-case interactor calls `author_repository.update(&mut tx, user_id, author)` inside a transaction
- **THEN** both the `author` UPDATE and the `event_set` + `author_event` INSERTs are committed together

### Requirement: mockall-generated mocks remain usable for unit tests
`BookRepository` and `AuthorRepository` traits with `#[automock]` and `&mut PgConnection` parameters SHALL still generate usable mock structs that can be injected into interactors in unit tests.

#### Scenario: Mock book repository with connection argument
- **WHEN** a unit test constructs `MockBookRepository` and sets `expect_create().with(always(), always(), always()).returning(|_, _, _| Ok(()))`
- **THEN** the interactor under test runs successfully without requiring a real database connection

#### Scenario: Mock author repository with connection argument
- **WHEN** a unit test constructs `MockAuthorRepository` and sets `expect_update().with(always(), always(), always()).returning(|_, _, _| Ok(()))`
- **THEN** the interactor under test runs successfully

### Requirement: Temporary ImportBooksRepository is removed
The `ImportBooksRepository` domain trait and `PgImportBooksRepository` infrastructure implementation SHALL be removed. The `ImportBooksInteractor` SHALL use the standard `BookRepository` and `AuthorRepository` traits, managing its own transaction via `PgPool`.

#### Scenario: Import books uses standard repositories
- **WHEN** `ImportBooksInteractor::import` is invoked
- **THEN** it begins a transaction, calls `author_repository.create(&mut tx, ...)` and `book_repository.create(&mut tx, ...)` for each book, and commits the transaction

#### Scenario: Import books rollback on partial failure
- **WHEN** `ImportBooksInteractor::import` is invoked and one book creation fails after several authors were created
- **THEN** the entire transaction is rolled back and no partial data remains in the database
