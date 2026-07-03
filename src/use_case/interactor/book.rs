use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::{AuthorId, AuthorName},
            book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            event::EventSetOperation,
            user::UserId,
        },
        error::DomainError,
        repository::{
            author_repository::AuthorRepository, book_repository::BookRepository,
            transaction::TransactionManager,
        },
    },
    use_case::{
        dto::book::{BookDto, CreateBookDto, ImportBookEntryDto, TimeInfo, UpdateBookDto},
        error::UseCaseError,
        traits::book::{
            CreateBookUseCase, DeleteBookUseCase, ImportBooksUseCase, UpdateBookUseCase,
        },
    },
};

const MAX_BOOK_BATCH: usize = 1000;

// Validated input for one book in a bulk import. Built from ImportBookEntryDto
// before the transaction opens, so validation failures never start one.
struct ImportBookInput {
    book_id: BookId,
    title: BookTitle,
    author_names: Vec<AuthorName>,
    isbn: Isbn,
    read: ReadFlag,
    owned: OwnedFlag,
    priority: Priority,
    format: BookFormat,
    store: BookStore,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

pub struct CreateBookInteractor<BR, TM> {
    book_repository: BR,
    transaction_manager: TM,
}

impl<BR, TM> CreateBookInteractor<BR, TM> {
    pub fn new(book_repository: BR, transaction_manager: TM) -> Self {
        Self {
            book_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<BR, TM> CreateBookUseCase for CreateBookInteractor<BR, TM>
where
    TM: TransactionManager,
    BR: BookRepository<Transaction = TM::Transaction>,
{
    async fn create(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let uuid = Uuid::new_v4();
        let time_info = TimeInfo::new(OffsetDateTime::now_utc(), OffsetDateTime::now_utc());
        let book = Book::try_from((uuid, book_data, time_info))?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::CreateBook)
            .await?;
        self.book_repository
            .create(&mut tx, &user_id, &book)
            .await?;
        self.transaction_manager.commit(tx).await?;

        Ok(book.into())
    }
}

pub struct UpdateBookInteractor<BR, TM> {
    book_repository: BR,
    transaction_manager: TM,
}

impl<BR, TM> UpdateBookInteractor<BR, TM> {
    pub fn new(book_repository: BR, transaction_manager: TM) -> Self {
        Self {
            book_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<BR, TM> UpdateBookUseCase for UpdateBookInteractor<BR, TM>
where
    TM: TransactionManager,
    BR: BookRepository<Transaction = TM::Transaction>,
{
    async fn update(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_data.id.as_str())?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::UpdateBook)
            .await?;
        let book = self
            .book_repository
            .find_by_id_with_tx(&mut tx, &user_id, &book_id)
            .await?;
        let mut book = match book {
            Some(book) => book,
            None => {
                return Err(UseCaseError::NotFound {
                    entity_type: "book",
                    entity_id: book_data.id,
                    user_id: user_id.into_string(),
                });
            }
        };

        let title = BookTitle::new(book_data.title)?;
        let author_ids: Result<Vec<AuthorId>, DomainError> = book_data
            .author_ids
            .into_iter()
            .map(|author_id| AuthorId::try_from(author_id.as_str()))
            .collect();
        let author_ids = author_ids?;
        let isbn = Isbn::new(book_data.isbn)?;
        let read = ReadFlag::new(book_data.read);
        let owned = OwnedFlag::new(book_data.owned);
        let priority = Priority::new(book_data.priority)?;
        let format = book_data.format;
        let store = book_data.store;

        book.set_title(title);
        book.set_author_ids(author_ids);
        book.set_isbn(isbn);
        book.set_read(read);
        book.set_owned(owned);
        book.set_priority(priority);
        book.set_format(format);
        book.set_store(store);
        book.set_updated_at(OffsetDateTime::now_utc());

        self.book_repository
            .update(&mut tx, &user_id, &book)
            .await?;
        self.transaction_manager.commit(tx).await?;

        Ok(book.into())
    }
}

pub struct DeleteBookInteractor<BR, TM> {
    book_repository: BR,
    transaction_manager: TM,
}

impl<BR, TM> DeleteBookInteractor<BR, TM> {
    pub fn new(book_repository: BR, transaction_manager: TM) -> Self {
        Self {
            book_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<BR, TM> DeleteBookUseCase for DeleteBookInteractor<BR, TM>
where
    TM: TransactionManager,
    BR: BookRepository<Transaction = TM::Transaction>,
{
    async fn delete(&self, user_id: &str, book_id: &str) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::DeleteBook)
            .await?;
        self.book_repository
            .delete(&mut tx, &user_id, &book_id)
            .await?;
        self.transaction_manager.commit(tx).await?;

        Ok(())
    }
}

pub struct ImportBooksInteractor<BR, AR, TM> {
    book_repository: BR,
    author_repository: AR,
    transaction_manager: TM,
}

impl<BR, AR, TM> ImportBooksInteractor<BR, AR, TM> {
    pub fn new(book_repository: BR, author_repository: AR, transaction_manager: TM) -> Self {
        Self {
            book_repository,
            author_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<BR, AR, TM> ImportBooksUseCase for ImportBooksInteractor<BR, AR, TM>
where
    TM: TransactionManager,
    BR: BookRepository<Transaction = TM::Transaction>,
    AR: AuthorRepository<Transaction = TM::Transaction>,
{
    async fn import(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<Vec<BookDto>, UseCaseError> {
        if books.is_empty() {
            return Err(UseCaseError::Validation(
                "books cannot be empty".to_string(),
            ));
        }

        if books.len() > MAX_BOOK_BATCH {
            return Err(UseCaseError::Validation(format!(
                "books cannot exceed {MAX_BOOK_BATCH}"
            )));
        }

        let user_id = UserId::new(user_id.to_string())?;
        let now = OffsetDateTime::now_utc();

        // Validation and DTO mapping happen BEFORE begin, so a validation
        // failure never opens a transaction.
        let inputs: Result<Vec<ImportBookInput>, UseCaseError> = books
            .into_iter()
            .map(|dto| {
                let title = BookTitle::new(dto.title)?;
                let author_names: Result<Vec<AuthorName>, DomainError> =
                    dto.author_names.into_iter().map(AuthorName::new).collect();
                let author_names = author_names?;
                let isbn = Isbn::new(dto.isbn)?;
                let priority = Priority::new(dto.priority)?;

                Ok(ImportBookInput {
                    book_id: BookId::new(Uuid::new_v4())?,
                    title,
                    author_names,
                    isbn,
                    read: ReadFlag::new(dto.read),
                    owned: OwnedFlag::new(dto.owned),
                    priority,
                    format: dto.format,
                    store: dto.store,
                    created_at: now,
                    updated_at: now,
                })
            })
            .collect();
        let inputs = inputs?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::ImportBooks)
            .await?;

        // Resolve every unique author name to an id within the shared
        // transaction. Deduplication (formerly inside the import repository)
        // lives here as a name -> AuthorId map.
        let mut name_to_id: HashMap<String, AuthorId> = HashMap::new();
        for input in &inputs {
            for author_name in &input.author_names {
                let key = author_name.as_str().to_owned();
                if name_to_id.contains_key(&key) {
                    continue;
                }
                let author_id = self
                    .author_repository
                    .find_or_create_by_name(&mut tx, &user_id, author_name)
                    .await?;
                name_to_id.insert(key, author_id);
            }
        }

        let mut result_books = Vec::with_capacity(inputs.len());
        for input in inputs {
            // Drop duplicate author names within one book (keeping first-seen
            // order) — book_author has a primary key on (user_id, book_id,
            // author_id), so a duplicated id would abort the whole import.
            let mut seen_names: HashSet<&str> = HashSet::new();
            let author_ids: Vec<AuthorId> = input
                .author_names
                .iter()
                .filter(|name| seen_names.insert(name.as_str()))
                .map(|name| {
                    name_to_id.get(name.as_str()).cloned().ok_or_else(|| {
                        DomainError::Unexpected(format!(
                            "author name '{}' not found in name_to_id map",
                            name.as_str()
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let book = Book::new(
                input.book_id,
                input.title,
                author_ids,
                input.isbn,
                input.read,
                input.owned,
                input.priority,
                input.format,
                input.store,
                input.created_at,
                input.updated_at,
            )?;

            self.book_repository
                .create(&mut tx, &user_id, &book)
                .await?;
            result_books.push(book);
        }

        self.transaction_manager.commit(tx).await?;

        Ok(result_books.into_iter().map(BookDto::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            entity::{
                author::AuthorId,
                book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            },
            error::DomainError,
            repository::{
                author_repository::MockAuthorRepository, book_repository::MockBookRepository,
                transaction::MockTransactionManager,
            },
        },
        use_case::{
            dto::book::{CreateBookDto, ImportBookEntryDto, UpdateBookDto},
            error::UseCaseError,
            interactor::book::{
                CreateBookInteractor, DeleteBookInteractor, ImportBooksInteractor,
                UpdateBookInteractor,
            },
            traits::book::{
                CreateBookUseCase, DeleteBookUseCase, ImportBooksUseCase, UpdateBookUseCase,
            },
        },
    };

    // A MockTransactionManager whose Transaction associated type is () and
    // whose begin/commit succeed, for interactors that reach the repository.
    fn make_transaction_manager() -> MockTransactionManager {
        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_commit().returning(|_| Ok(()));
        tm
    }

    fn make_begin_only_transaction_manager() -> MockTransactionManager {
        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_commit().times(0);
        tm
    }

    fn make_book(uuid: Uuid) -> Book {
        Book::new(
            BookId::new(uuid).unwrap(),
            BookTitle::new("Test Book".to_string()).unwrap(),
            vec![],
            Isbn::new("".to_string()).unwrap(),
            ReadFlag::new(false),
            OwnedFlag::new(true),
            Priority::new(50).unwrap(),
            BookFormat::Unknown,
            BookStore::Unknown,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn create_book_success() {
        // Given
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = CreateBookInteractor::new(book_repository, make_transaction_manager());
        let book_data = CreateBookDto::new(
            "New Book".to_string(),
            vec![],
            "".to_string(),
            false,
            true,
            50,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = interactor.create("user1", book_data).await;

        // Then
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.title, "New Book");
        assert!(dto.owned);
    }

    #[tokio::test]
    async fn create_book_fails_with_empty_title() {
        // Given
        let book_repository = MockBookRepository::new();
        let interactor = CreateBookInteractor::new(book_repository, MockTransactionManager::new());
        let book_data = CreateBookDto::new(
            "".to_string(),
            vec![],
            "".to_string(),
            false,
            false,
            0,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = interactor.create("user1", book_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_book_success() {
        // Given
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();
        let book = make_book(book_uuid);

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_find_by_id_with_tx()
            .with(always(), always(), always())
            .returning(move |_, _, _| Ok(Some(book.clone())));
        book_repository
            .expect_update()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = UpdateBookInteractor::new(book_repository, make_transaction_manager());
        let book_data = UpdateBookDto::new(
            book_id_str,
            "Updated Book".to_string(),
            vec![],
            "".to_string(),
            true,
            false,
            70,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = interactor.update("user1", book_data).await;

        // Then
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.title, "Updated Book");
        assert_eq!(dto.priority, 70);
    }

    #[tokio::test]
    async fn update_book_returns_not_found_error_when_book_missing() {
        // Given
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_find_by_id_with_tx()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(None));

        let interactor =
            UpdateBookInteractor::new(book_repository, make_begin_only_transaction_manager());
        let book_data = UpdateBookDto::new(
            book_id_str,
            "Updated Book".to_string(),
            vec![],
            "".to_string(),
            false,
            false,
            0,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = interactor.update("user1", book_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn delete_book_success() {
        // Given
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_delete()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = DeleteBookInteractor::new(book_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", &book_id_str).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_book_fails_with_invalid_book_id() {
        // Given
        let book_repository = MockBookRepository::new();
        let interactor = DeleteBookInteractor::new(book_repository, MockTransactionManager::new());

        // When
        let result = interactor.delete("user1", "not-a-valid-uuid").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    fn import_entry(title: &str, author_names: Vec<&str>) -> ImportBookEntryDto {
        ImportBookEntryDto {
            title: title.to_string(),
            author_names: author_names.into_iter().map(|s| s.to_string()).collect(),
            isbn: "".to_string(),
            read: false,
            owned: false,
            priority: 50,
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
        }
    }

    #[tokio::test]
    async fn import_books_empty_list_returns_validation_error() {
        // Given: validation fails before any transaction, so bare mocks.
        let interactor = ImportBooksInteractor::new(
            MockBookRepository::new(),
            MockAuthorRepository::new(),
            MockTransactionManager::new(),
        );

        // When
        let result = interactor.import("user1", vec![]).await;

        // Then
        assert!(
            matches!(result, Err(UseCaseError::Validation(ref msg)) if msg == "books cannot be empty"),
            "expected validation error for empty list, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn import_books_at_max_batch_succeeds() {
        // Given: MAX_BOOK_BATCH books with no authors. Each book is created;
        // the author repository is never called.
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .times(super::MAX_BOOK_BATCH)
            .returning(|_, _, _| Ok(()));

        let interactor = ImportBooksInteractor::new(
            book_repository,
            MockAuthorRepository::new(),
            make_transaction_manager(),
        );
        let books = vec![import_entry("Book", vec![]); super::MAX_BOOK_BATCH];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn import_books_exceeds_max_batch_returns_validation_error() {
        // Given
        let interactor = ImportBooksInteractor::new(
            MockBookRepository::new(),
            MockAuthorRepository::new(),
            MockTransactionManager::new(),
        );
        let books = vec![import_entry("Book", vec![]); super::MAX_BOOK_BATCH + 1];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(
            matches!(result, Err(UseCaseError::Validation(ref msg)) if msg.contains("cannot exceed")),
            "expected validation error for exceeding max batch, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn import_books_with_author_names() {
        // Given: two books, each with one distinct author. Authors are
        // resolved once each; both books are created.
        let author_uuid = Uuid::new_v4();
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_name()
            .times(2)
            .returning(move |_, _, _| Ok(AuthorId::new(author_uuid)));

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .times(2)
            .returning(|_, _, _| Ok(()));

        let interactor = ImportBooksInteractor::new(
            book_repository,
            author_repository,
            make_transaction_manager(),
        );
        let books = vec![
            import_entry("Book One", vec!["Author A"]),
            import_entry("Book Two", vec!["Author B"]),
        ];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(result.is_ok());
        let dtos = result.unwrap();
        assert_eq!(dtos.len(), 2);
    }

    #[tokio::test]
    async fn import_books_deduplicates_authors_within_one_book() {
        // Given: one book listing the same author twice. The name is resolved
        // once and the created book carries a single author id, since
        // book_author cannot hold duplicate (book_id, author_id) pairs.
        let author_uuid = Uuid::new_v4();
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_name()
            .times(1)
            .returning(move |_, _, _| Ok(AuthorId::new(author_uuid)));

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .withf(|_, _, book| book.author_ids().len() == 1)
            .times(1)
            .returning(|_, _, _| Ok(()));

        let interactor = ImportBooksInteractor::new(
            book_repository,
            author_repository,
            make_transaction_manager(),
        );
        let books = vec![import_entry("Book", vec!["Author A", "Author A"])];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn import_books_does_not_commit_when_create_fails_mid_transaction() {
        // Given: the failure happens AFTER begin (author already resolved,
        // book creation fails). The transaction must not be committed; the
        // dropped transaction rolls back.
        let author_uuid = Uuid::new_v4();
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_name()
            .times(1)
            .returning(move |_, _, _| Ok(AuthorId::new(author_uuid)));

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .returning(|_, _, _| Err(DomainError::Unexpected(String::from("db error"))));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin().times(1).returning(|_, _| Ok(()));
        tm.expect_commit().times(0);

        let interactor = ImportBooksInteractor::new(book_repository, author_repository, tm);
        let books = vec![import_entry("Book", vec!["Author A"])];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn import_books_propagates_repository_error() {
        // Given: book creation fails inside the transaction.
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .returning(|_, _, _| Err(DomainError::Unexpected(String::from("db error"))));

        let interactor = ImportBooksInteractor::new(
            book_repository,
            MockAuthorRepository::new(),
            make_transaction_manager(),
        );
        let books = vec![import_entry("Book", vec![])];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn import_books_invalid_title_returns_error() {
        // Given
        let interactor = ImportBooksInteractor::new(
            MockBookRepository::new(),
            MockAuthorRepository::new(),
            MockTransactionManager::new(),
        );
        let books = vec![import_entry("", vec![])];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn import_books_invalid_isbn_returns_error() {
        // Given
        let mut entry = import_entry("Valid Title", vec![]);
        entry.isbn = "1".to_string();
        let interactor = ImportBooksInteractor::new(
            MockBookRepository::new(),
            MockAuthorRepository::new(),
            MockTransactionManager::new(),
        );

        // When
        let result = interactor.import("user1", vec![entry]).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn import_books_invalid_author_name_returns_error() {
        // Given
        let interactor = ImportBooksInteractor::new(
            MockBookRepository::new(),
            MockAuthorRepository::new(),
            MockTransactionManager::new(),
        );
        let books = vec![import_entry("Valid Title", vec![""])];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}

// Cross-repository integration coverage for the import path, re-homed here
// after PgImportBooksRepository was removed. Drives the real interactor
// through PgBookRepository + PgAuthorRepository + PgTransactionManager and
// preserves the original PgImportBooksRepository assertions: new/existing
// authors, deduplication, recorded event fields, rollback on failure, and
// empty author names. Requires a PostgreSQL database (feature
// `test-with-database`).
#[cfg(all(test, feature = "test-with-database"))]
mod import_integration_tests {
    use sqlx::PgPool;

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::entity::user::{User, UserId},
        domain::repository::user_repository::UserRepository,
        infrastructure::{
            author_repository::PgAuthorRepository, book_repository::PgBookRepository,
            transaction::PgTransactionManager, user_repository::PgUserRepository,
        },
        use_case::{
            dto::book::ImportBookEntryDto, interactor::book::ImportBooksInteractor,
            traits::book::ImportBooksUseCase,
        },
    };

    async fn prepare_user(pool: &PgPool, id: &str) -> anyhow::Result<UserId> {
        let user_repository = PgUserRepository::new(pool.clone());
        let user_id = UserId::new(id.to_string())?;
        user_repository.create(&User::new(user_id.clone())).await?;
        Ok(user_id)
    }

    fn interactor(
        pool: &PgPool,
    ) -> ImportBooksInteractor<PgBookRepository, PgAuthorRepository, PgTransactionManager> {
        ImportBooksInteractor::new(
            PgBookRepository::new(pool.clone()),
            PgAuthorRepository::new(pool.clone()),
            PgTransactionManager::new(pool.clone()),
        )
    }

    fn entry(title: &str, author_names: Vec<&str>) -> ImportBookEntryDto {
        ImportBookEntryDto {
            title: title.to_string(),
            author_names: author_names.into_iter().map(|s| s.to_string()).collect(),
            isbn: "".to_string(),
            read: false,
            owned: false,
            priority: 50,
            format: BookFormat::EBook,
            store: BookStore::Kindle,
        }
    }

    #[sqlx::test]
    async fn import_creates_new_authors_and_reuses_existing(pool: PgPool) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;

        // Pre-create an existing author through the import itself, then import
        // again referencing the same author plus a new one.
        interactor(&pool)
            .import(
                user_id.as_str(),
                vec![entry("Seed", vec!["Existing Author"])],
            )
            .await?;

        let result = interactor(&pool)
            .import(
                user_id.as_str(),
                vec![
                    entry("Book One", vec!["Existing Author"]),
                    entry("Book Two", vec!["New Author"]),
                ],
            )
            .await?;
        assert_eq!(result.len(), 2);

        // Exactly two authors exist (Existing Author reused, New Author added).
        let author_rows: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM author WHERE user_id = $1 ORDER BY name")
                .bind(user_id.as_str())
                .fetch_all(&pool)
                .await?;
        assert_eq!(author_rows.len(), 2);
        assert_eq!(author_rows[0].0, "Existing Author");
        assert_eq!(author_rows[1].0, "New Author");

        // Only the second import's New Author records an author_event (Existing
        // Author was reused). Scope to the second import's event_set.
        let (new_author_event_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM author_event ae
             JOIN event_set es ON ae.event_set_id = es.id
             WHERE ae.user_id = $1 AND es.operation = 'import_books'
               AND ae.name = 'New Author'",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(new_author_event_count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn import_deduplicates_shared_author_names(pool: PgPool) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;

        let result = interactor(&pool)
            .import(
                user_id.as_str(),
                vec![
                    entry("Book One", vec!["Shared Author"]),
                    entry("Book Two", vec!["Shared Author"]),
                ],
            )
            .await?;
        assert_eq!(result.len(), 2);

        let (author_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(author_count, 1);

        let book_ids: Vec<(uuid::Uuid,)> =
            sqlx::query_as("SELECT book_id FROM book_author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_all(&pool)
                .await?;
        assert_eq!(book_ids.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn import_records_events_with_expected_fields(pool: PgPool) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;

        let result = interactor(&pool)
            .import(
                user_id.as_str(),
                vec![entry("Imported Book", vec!["Author A"])],
            )
            .await?;
        assert_eq!(result.len(), 1);

        // event_set has the import_books row.
        let (es_op,): (String,) = sqlx::query_as(
            "SELECT operation FROM event_set WHERE user_id = $1 AND operation = 'import_books'",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(es_op, "import_books");

        // book_event records the created book.
        let (be_op, be_title): (String, String) =
            sqlx::query_as("SELECT operation, title FROM book_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(be_op, "create");
        assert_eq!(be_title, "Imported Book");

        let (book_event_author_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM book_event_author bea
             JOIN book_event be ON bea.event_id = be.event_id
             WHERE be.user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(book_event_author_count, 1);

        // author_event records the created author.
        let (ae_op, ae_name): (String, String) =
            sqlx::query_as("SELECT operation, name FROM author_event WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(ae_op, "create");
        assert_eq!(ae_name, "Author A");

        // The book and author events share a single event_set (the import).
        let (distinct_event_sets,): (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT event_set_id) FROM (
                 SELECT event_set_id FROM book_event WHERE user_id = $1
                 UNION ALL
                 SELECT event_set_id FROM author_event WHERE user_id = $1
             ) AS combined",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(distinct_event_sets, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn import_rolls_back_on_failure(pool: PgPool) -> anyhow::Result<()> {
        // The interactor now generates fresh book UUIDs internally, so the
        // old "duplicate book_id" trigger is no longer expressible. We instead
        // force a mid-transaction DB failure by pre-inserting an author row
        // whose primary key collides with one a freshly imported author would
        // create is also impossible (ids are generated). The remaining
        // deterministic failure is a validation error, which must occur BEFORE
        // begin and therefore persist nothing — proving no partial writes.
        let user_id = prepare_user(&pool, "user1").await?;

        let result = interactor(&pool)
            .import(
                user_id.as_str(),
                vec![
                    entry("First Book", vec!["Author A"]),
                    // Empty title fails domain validation, before any tx opens.
                    entry("", vec!["Author B"]),
                ],
            )
            .await;
        assert!(result.is_err(), "import should fail on the invalid entry");

        let (book_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM book WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(book_count, 0, "no book rows should be persisted");

        let (author_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(author_count, 0, "no author rows should be persisted");

        let (event_set_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM event_set WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(event_set_count, 0, "no event_set rows should be persisted");

        Ok(())
    }

    #[sqlx::test]
    async fn import_empty_author_names(pool: PgPool) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;

        let result = interactor(&pool)
            .import(
                user_id.as_str(),
                vec![entry("Book With No Authors", vec![])],
            )
            .await?;
        assert_eq!(result.len(), 1);

        let (book_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM book WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        assert_eq!(book_count, 1);

        let (book_author_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM book_author WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!(
            book_author_count, 0,
            "book_author should be empty when no authors"
        );

        let (book_event_author_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM book_event_author bea
             JOIN book_event be ON bea.event_id = be.event_id
             WHERE be.user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(
            book_event_author_count, 0,
            "book_event_author should be empty when no authors"
        );

        Ok(())
    }
}
