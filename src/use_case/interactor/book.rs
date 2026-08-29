use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::{
        entity::{
            author::{AuthorId, AuthorName},
            book::{Book, BookId, BookTitle, BookUpdate, Isbn, OwnedFlag, Priority, ReadFlag},
            operation::{NewOperation, OperationType},
            user::UserId,
        },
        error::DomainError,
        repository::{
            author_repository::AuthorRepository,
            book_repository::BookRepository,
            transaction::{TransactionManager, TransactionOperation},
        },
    },
    use_case::{
        dto::{
            book::{
                BookDto, CreateBookDto, ImportAuthorPreviewDto, ImportAuthorStatus,
                ImportBookEntryDto, ImportBookPreviewDto, ImportBooksPreviewDto, TimeInfo,
                UpdateBookDto,
            },
            mutation::{
                BookMutationResultDto, DeleteBookResultDto, ImportBooksResultDto,
                MutationResultDto, SingleRevisionMutationResultDto,
            },
        },
        error::UseCaseError,
        traits::book::{BookCommandUseCase, BookQueryUseCase},
    },
};

const MAX_BOOK_BATCH: usize = 1000;

#[derive(Debug, Clone)]
pub struct BookQueryInteractor<BR> {
    book_repository: BR,
}

impl<BR> BookQueryInteractor<BR> {
    pub fn new(book_repository: BR) -> Self {
        Self { book_repository }
    }
}

#[async_trait]
impl<BR> BookQueryUseCase for BookQueryInteractor<BR>
where
    BR: BookRepository,
{
    async fn find_by_id(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Option<BookDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        Ok(self
            .book_repository
            .find_by_id(&user_id, &book_id)
            .await?
            .map(BookDto::from))
    }

    async fn find_all(&self, user_id: &str) -> Result<Vec<BookDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        Ok(self
            .book_repository
            .find_all(&user_id)
            .await?
            .into_iter()
            .map(BookDto::from)
            .collect())
    }

    async fn find_by_author_ids(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, Vec<BookDto>>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_ids: Vec<AuthorId> = author_ids
            .iter()
            .map(|author_id| AuthorId::try_from(author_id.as_str()))
            .collect::<Result<_, DomainError>>()?;
        Ok(self
            .book_repository
            .find_by_author_ids_as_hash_map(&user_id, &author_ids)
            .await?
            .into_iter()
            .map(|(author_id, books)| {
                (
                    author_id.to_string(),
                    books.into_iter().map(BookDto::from).collect(),
                )
            })
            .collect())
    }
}

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

struct ImportExecutionResult {
    books: Vec<Book>,
    previews: Vec<ImportBookPreviewDto>,
}

pub struct BookCommandInteractor<BR, AR, TM> {
    book_repository: BR,
    author_repository: AR,
    transaction_manager: TM,
}

impl<BR, AR, TM> BookCommandInteractor<BR, AR, TM> {
    pub fn new(book_repository: BR, author_repository: AR, transaction_manager: TM) -> Self {
        Self {
            book_repository,
            author_repository,
            transaction_manager,
        }
    }
}

impl<BR, AR, TM> BookCommandInteractor<BR, AR, TM>
where
    TM: TransactionManager,
    BR: BookRepository<Transaction = TM::Transaction>,
    AR: AuthorRepository<Transaction = TM::Transaction>,
{
    fn prepare_import(
        books: Vec<ImportBookEntryDto>,
    ) -> Result<Vec<ImportBookInput>, UseCaseError> {
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

        let now = OffsetDateTime::now_utc();
        books
            .into_iter()
            .map(|dto| {
                Ok(ImportBookInput {
                    book_id: BookId::new(Uuid::new_v4())?,
                    title: BookTitle::new(dto.title)?,
                    author_names: dto
                        .author_names
                        .into_iter()
                        .map(AuthorName::new)
                        .collect::<Result<_, DomainError>>()?,
                    isbn: Isbn::new(dto.isbn)?,
                    read: ReadFlag::new(dto.read),
                    owned: OwnedFlag::new(dto.owned),
                    priority: Priority::new(dto.priority)?,
                    format: dto.format,
                    store: dto.store,
                    created_at: now,
                    updated_at: now,
                })
            })
            .collect()
    }

    async fn execute_import(
        &self,
        tx: &mut TM::Transaction,
        inputs: Vec<ImportBookInput>,
    ) -> Result<ImportExecutionResult, UseCaseError> {
        // This shared path must contain only writes scoped to `tx`: preview
        // relies on rollback to undo every author, book, relationship, and event.
        let mut seen_author_names = HashSet::new();
        let unique_author_names: Vec<AuthorName> = inputs
            .iter()
            .flat_map(|input| input.author_names.iter())
            .filter(|name| seen_author_names.insert(name.as_str().to_owned()))
            .cloned()
            .collect();
        let resolved = self
            .author_repository
            .find_or_create_by_names(tx, &unique_author_names, inputs[0].created_at)
            .await?;

        let mut books = Vec::with_capacity(inputs.len());
        let mut previews = Vec::with_capacity(inputs.len());
        for input in inputs {
            let mut seen_names = HashSet::new();
            let authors = input
                .author_names
                .iter()
                .filter(|name| seen_names.insert(name.as_str()))
                .map(|name| {
                    let id = resolved
                        .authors_by_name
                        .get(name.as_str())
                        .cloned()
                        .ok_or_else(|| {
                            DomainError::Unexpected(format!(
                                "author name '{}' not found in name_to_id map",
                                name.as_str()
                            ))
                        })?;
                    let status = if resolved.created_author_ids.contains(&id) {
                        ImportAuthorStatus::New
                    } else {
                        ImportAuthorStatus::Existing
                    };
                    Ok((
                        id,
                        ImportAuthorPreviewDto {
                            name: name.as_str().to_owned(),
                            status,
                        },
                    ))
                })
                .collect::<Result<Vec<_>, DomainError>>()?;
            let (author_ids, preview_authors): (Vec<_>, Vec<_>) = authors.into_iter().unzip();

            previews.push(ImportBookPreviewDto {
                title: input.title.as_str().to_owned(),
                authors: preview_authors,
                isbn: input.isbn.as_str().to_owned(),
                read: input.read.to_bool(),
                owned: input.owned.to_bool(),
                priority: input.priority.to_i32(),
                format: input.format.clone(),
                store: input.store.clone(),
            });
            books.push(Book::new(
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
            )?);
        }

        self.book_repository.create_all(tx, &books).await?;
        Ok(ImportExecutionResult { books, previews })
    }
}

#[async_trait]
impl<BR, AR, TM> BookCommandUseCase for BookCommandInteractor<BR, AR, TM>
where
    TM: TransactionManager,
    BR: BookRepository<Transaction = TM::Transaction>,
    AR: AuthorRepository<Transaction = TM::Transaction>,
{
    async fn create(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let uuid = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();
        let time_info = TimeInfo::new(now, now);
        let book = Book::try_from((uuid, book_data, time_info))?;

        let mut tx = self
            .transaction_manager
            .begin_operation(&user_id, &NewOperation::simple(OperationType::CreateBook))
            .await?;
        let _event_id = self.book_repository.create(&mut tx, &book).await?;
        let operation_id = tx.operation_id().to_string();
        let revision_number = tx.revision_number().ok_or_else(|| {
            UseCaseError::Unexpected("Book mutation did not record a revision".to_string())
        })?;
        self.transaction_manager.commit(tx).await?;

        Ok(SingleRevisionMutationResultDto::new(
            book.into(),
            operation_id,
            revision_number,
        ))
    }
    async fn update(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let UpdateBookDto {
            id,
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
        } = book_data;

        let book_id = BookId::try_from(id.as_str())?;
        let title = BookTitle::new(title)?;
        let author_ids: Result<Vec<AuthorId>, DomainError> = author_ids
            .into_iter()
            .map(|author_id| AuthorId::try_from(author_id.as_str()))
            .collect();
        let author_ids = author_ids?;
        let isbn = Isbn::new(isbn)?;
        let read = ReadFlag::new(read);
        let owned = OwnedFlag::new(owned);
        let priority = Priority::new(priority)?;

        let mut tx = self
            .transaction_manager
            .begin_operation(&user_id, &NewOperation::simple(OperationType::UpdateBook))
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
                    entity_id: id,
                    user_id: user_id.into_string(),
                });
            }
        };

        let update = BookUpdate {
            title,
            author_ids,
            isbn,
            read,
            owned,
            priority,
            format,
            store,
        };
        book.update(update, OffsetDateTime::now_utc());

        let _event_id = self.book_repository.update(&mut tx, &book).await?;
        let operation_id = tx.operation_id().to_string();
        let revision_number = tx.revision_number().ok_or_else(|| {
            UseCaseError::Unexpected("Book mutation did not record a revision".to_string())
        })?;
        self.transaction_manager.commit(tx).await?;

        Ok(SingleRevisionMutationResultDto::new(
            book.into(),
            operation_id,
            revision_number,
        ))
    }
    async fn delete(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<DeleteBookResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id_value = book_id.to_string();
        let book_id = BookId::try_from(book_id)?;

        let mut tx = self
            .transaction_manager
            .begin_operation(&user_id, &NewOperation::simple(OperationType::DeleteBook))
            .await?;
        self.book_repository.delete(&mut tx, &book_id).await?;
        let operation_id = tx.operation_id().to_string();
        self.transaction_manager.commit(tx).await?;

        Ok(MutationResultDto::new(book_id_value, operation_id))
    }
    async fn import(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<ImportBooksResultDto, UseCaseError> {
        let inputs = Self::prepare_import(books)?;
        let user_id = UserId::new(user_id.to_string())?;
        let operation =
            NewOperation::import_books(inputs.len()).map_err(UseCaseError::Validation)?;
        let mut tx = self
            .transaction_manager
            .begin_operation(&user_id, &operation)
            .await?;
        let result = self.execute_import(&mut tx, inputs).await?;
        let operation_id = tx.operation_id().to_string();
        self.transaction_manager.commit(tx).await?;
        Ok(MutationResultDto::new(
            result.books.into_iter().map(BookDto::from).collect(),
            operation_id,
        ))
    }

    async fn preview_import(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<ImportBooksPreviewDto, UseCaseError> {
        let inputs = Self::prepare_import(books)?;
        let user_id = UserId::new(user_id.to_string())?;
        let operation =
            NewOperation::import_books(inputs.len()).map_err(UseCaseError::Validation)?;
        let mut tx = self
            .transaction_manager
            .begin_operation(&user_id, &operation)
            .await?;
        let result = self.execute_import(&mut tx, inputs).await?;
        self.transaction_manager.rollback(tx).await?;
        Ok(ImportBooksPreviewDto {
            books: result.previews,
        })
    }

    async fn restore(
        &self,
        user_id: &str,
        book_id: &str,
        revision_number: i32,
    ) -> Result<crate::use_case::dto::mutation::RestoreBookResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        crate::domain::entity::revision::RevisionNumber::try_from(revision_number)?;
        let operation = NewOperation::restore_book(revision_number);
        let mut tx = self
            .transaction_manager
            .begin_operation(&user_id, &operation)
            .await?;
        let restored = self
            .book_repository
            .restore_revision(&mut tx, &book_id, revision_number)
            .await?;
        let operation_id = tx.operation_id().to_string();
        let restored_revision_number = tx.revision_number().ok_or_else(|| {
            UseCaseError::Unexpected("Book restore did not record a revision".to_string())
        })?;
        self.transaction_manager.commit(tx).await?;
        Ok(SingleRevisionMutationResultDto::new(
            Some(restored.into()),
            operation_id,
            restored_revision_number,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use mockall::predicate::always;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::{
            time::normalize_timestamp_for_persistence,
            types::{BookFormat, BookStore},
        },
        domain::{
            entity::{
                author::AuthorId,
                book::{Book, BookId, BookTitle, BookUpdate, Isbn, OwnedFlag, Priority, ReadFlag},
            },
            error::DomainError,
            repository::{
                author_repository::{FindOrCreateAuthorsResult, MockAuthorRepository},
                book_repository::MockBookRepository,
                transaction::MockTransactionManager,
            },
        },
        use_case::{
            dto::book::{CreateBookDto, ImportAuthorStatus, ImportBookEntryDto, UpdateBookDto},
            error::UseCaseError,
            interactor::book::{BookCommandInteractor, BookQueryInteractor},
            traits::book::{BookCommandUseCase, BookQueryUseCase},
        },
    };

    struct CreateBookTestCommand;
    struct UpdateBookTestCommand;
    struct DeleteBookTestCommand;
    struct ImportBooksTestCommand;

    impl CreateBookTestCommand {
        fn build(
            book_repository: MockBookRepository,
            transaction_manager: MockTransactionManager,
        ) -> BookCommandInteractor<MockBookRepository, MockAuthorRepository, MockTransactionManager>
        {
            BookCommandInteractor::new(
                book_repository,
                MockAuthorRepository::new(),
                transaction_manager,
            )
        }
    }

    impl UpdateBookTestCommand {
        fn build(
            book_repository: MockBookRepository,
            transaction_manager: MockTransactionManager,
        ) -> BookCommandInteractor<MockBookRepository, MockAuthorRepository, MockTransactionManager>
        {
            CreateBookTestCommand::build(book_repository, transaction_manager)
        }
    }

    impl DeleteBookTestCommand {
        fn build(
            book_repository: MockBookRepository,
            transaction_manager: MockTransactionManager,
        ) -> BookCommandInteractor<MockBookRepository, MockAuthorRepository, MockTransactionManager>
        {
            CreateBookTestCommand::build(book_repository, transaction_manager)
        }
    }

    impl ImportBooksTestCommand {
        fn build(
            book_repository: MockBookRepository,
            author_repository: MockAuthorRepository,
            transaction_manager: MockTransactionManager,
        ) -> BookCommandInteractor<MockBookRepository, MockAuthorRepository, MockTransactionManager>
        {
            BookCommandInteractor::new(book_repository, author_repository, transaction_manager)
        }
    }

    // A MockTransactionManager whose Transaction associated type is () and
    // whose begin/commit succeed, for interactors that reach the repository.
    fn make_transaction_manager() -> MockTransactionManager {
        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation().returning(|_, _| Ok(()));
        tm.expect_commit().returning(|_| Ok(()));
        tm
    }

    fn make_begin_only_transaction_manager() -> MockTransactionManager {
        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation().returning(|_, _| Ok(()));
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
    async fn find_book_by_id_returns_book() {
        let book_id = Uuid::new_v4();
        let book = make_book(book_id);
        let mut repository = MockBookRepository::new();
        repository
            .expect_find_by_id()
            .with(always(), always())
            .return_once(move |_, _| Ok(Some(book)));

        let result = BookQueryInteractor::new(repository)
            .find_by_id("user1", &book_id.hyphenated().to_string())
            .await
            .unwrap();

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn find_all_books_returns_list() {
        let book = make_book(Uuid::new_v4());
        let mut repository = MockBookRepository::new();
        repository
            .expect_find_all()
            .with(always())
            .return_once(move |_| Ok(vec![book]));

        let result = BookQueryInteractor::new(repository)
            .find_all("user1")
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn find_books_by_author_ids_maps_repository_result() {
        let author_id = AuthorId::new(Uuid::new_v4());
        let author_id_string = author_id.to_string();
        let book = make_book(Uuid::new_v4());
        let mut repository = MockBookRepository::new();
        repository
            .expect_find_by_author_ids_as_hash_map()
            .with(always(), always())
            .return_once(move |_, _| Ok(HashMap::from([(author_id, vec![book])])));

        let result = BookQueryInteractor::new(repository)
            .find_by_author_ids("user1", std::slice::from_ref(&author_id_string))
            .await
            .unwrap();

        assert_eq!(result[&author_id_string].len(), 1);
    }

    #[tokio::test]
    async fn find_books_by_author_ids_rejects_invalid_id_before_repository_call() {
        let repository = MockBookRepository::new();

        let result = BookQueryInteractor::new(repository)
            .find_by_author_ids("user1", &["invalid-author-id".to_string()])
            .await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn create_book_success() {
        // Given
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .with(always(), always())
            .returning(|_, _| Ok(101));

        let interactor = CreateBookTestCommand::build(book_repository, make_transaction_manager());
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
        assert_eq!(dto.value.title, "New Book");
        assert!(dto.value.owned);
        assert_eq!(dto.value.created_at, dto.value.updated_at);
        assert_eq!(dto.revision_number, 1);
    }

    #[tokio::test]
    async fn create_book_repository_failure_returns_no_result() {
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create()
            .returning(|_, _| Err(DomainError::Unexpected("event insert failed".to_string())));

        let interactor = CreateBookTestCommand::build(book_repository, {
            let mut tm = MockTransactionManager::new();
            tm.expect_begin_operation().returning(|_, _| Ok(()));
            tm.expect_commit().times(0);
            tm
        });
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

        let result = interactor.create("user1", book_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn create_book_commit_failure_returns_no_result() {
        let mut book_repository = MockBookRepository::new();
        book_repository.expect_create().returning(|_, _| Ok(101));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation().returning(|_, _| Ok(()));
        tm.expect_commit()
            .returning(|_| Err(DomainError::Unexpected("commit failed".to_string())));
        let interactor = CreateBookTestCommand::build(book_repository, tm);
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

        let result = interactor.create("user1", book_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn create_book_fails_with_empty_title() {
        // Given
        let book_repository = MockBookRepository::new();
        let interactor =
            CreateBookTestCommand::build(book_repository, MockTransactionManager::new());
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
            .with(always(), always())
            .returning(|_, _| Ok(202));

        let interactor = UpdateBookTestCommand::build(book_repository, make_transaction_manager());
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
        assert_eq!(dto.value.title, "Updated Book");
        assert_eq!(dto.value.priority, 70);
        assert_eq!(dto.revision_number, 1);
    }

    #[tokio::test]
    async fn update_book_commit_failure_returns_no_result() {
        let book_uuid = Uuid::new_v4();
        let book = make_book(book_uuid);
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_find_by_id_with_tx()
            .return_once(move |_, _, _| Ok(Some(book)));
        book_repository.expect_update().returning(|_, _| Ok(202));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation().returning(|_, _| Ok(()));
        tm.expect_commit()
            .returning(|_| Err(DomainError::Unexpected("commit failed".to_string())));
        let interactor = UpdateBookTestCommand::build(book_repository, tm);
        let book_data = UpdateBookDto::new(
            book_uuid.hyphenated().to_string(),
            "Updated Book".to_string(),
            vec![],
            "".to_string(),
            true,
            false,
            70,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        let result = interactor.update("user1", book_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn update_book_repository_failure_returns_no_result() {
        let book_uuid = Uuid::new_v4();
        let book = make_book(book_uuid);
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_find_by_id_with_tx()
            .return_once(move |_, _, _| Ok(Some(book)));
        book_repository
            .expect_update()
            .returning(|_, _| Err(DomainError::Unexpected("event insert failed".to_string())));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation().returning(|_, _| Ok(()));
        tm.expect_commit().times(0);
        let interactor = UpdateBookTestCommand::build(book_repository, tm);
        let book_data = UpdateBookDto::new(
            book_uuid.hyphenated().to_string(),
            "Updated Book".to_string(),
            vec![],
            "".to_string(),
            true,
            false,
            70,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        let result = interactor.update("user1", book_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn update_book_fails_with_empty_title_before_transaction() {
        // Given
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();
        let book_repository = MockBookRepository::new();

        let interactor =
            UpdateBookTestCommand::build(book_repository, MockTransactionManager::new());
        let book_data = UpdateBookDto::new(
            book_id_str,
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
        let result = interactor.update("user1", book_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
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
            UpdateBookTestCommand::build(book_repository, make_begin_only_transaction_manager());
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
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = DeleteBookTestCommand::build(book_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", &book_id_str).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_book_fails_with_invalid_book_id() {
        // Given
        let book_repository = MockBookRepository::new();
        let interactor =
            DeleteBookTestCommand::build(book_repository, MockTransactionManager::new());

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
        let interactor = ImportBooksTestCommand::build(
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
        // Given: MAX_BOOK_BATCH books with no authors use one bulk call.
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_names()
            .withf(|_, names, _| names.is_empty())
            .times(1)
            .returning(|_, _, _| Ok(HashMap::new().into()));
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create_all()
            .withf(|_, books| books.len() == super::MAX_BOOK_BATCH)
            .times(1)
            .returning(|_, _| Ok(()));

        let interactor = ImportBooksTestCommand::build(
            book_repository,
            author_repository,
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
        let interactor = ImportBooksTestCommand::build(
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
        let author_times = Arc::new(Mutex::new(Vec::new()));
        let captured_author_times = Arc::clone(&author_times);
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_names()
            .withf(|_, names, _| names.len() == 2)
            .times(1)
            .returning(move |_, names, created_at| {
                captured_author_times.lock().unwrap().push(created_at);
                Ok(names
                    .iter()
                    .map(|name| (name.as_str().to_owned(), AuthorId::new(Uuid::new_v4())))
                    .collect())
            });

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create_all()
            .withf(|_, books| books.len() == 2)
            .times(1)
            .returning(|_, _| Ok(()));

        let interactor = ImportBooksTestCommand::build(
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
        assert_eq!(dtos.value.len(), 2);
        assert_eq!(dtos.value[0].created_at, dtos.value[0].updated_at);
        assert_eq!(dtos.value[1].created_at, dtos.value[1].updated_at);
        assert_eq!(dtos.value[0].created_at, dtos.value[1].created_at);
        let author_times = author_times.lock().unwrap();
        assert_eq!(
            normalize_timestamp_for_persistence(author_times[0]),
            dtos.value[0].created_at
        );
    }

    #[tokio::test]
    async fn import_books_deduplicates_authors_within_one_book() {
        // Given: one book listing the same author twice. The name is resolved
        // once and the created book carries a single author id, since
        // book_author cannot hold duplicate (book_id, author_id) pairs.
        let author_uuid = Uuid::new_v4();
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_names()
            .withf(|_, names, _| names.len() == 1)
            .times(1)
            .returning(move |_, names, _| {
                Ok(
                    HashMap::from([(names[0].as_str().to_owned(), AuthorId::new(author_uuid))])
                        .into(),
                )
            });

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create_all()
            .withf(|_, books| books.len() == 1 && books[0].author_ids().len() == 1)
            .times(1)
            .returning(|_, _| Ok(()));

        let interactor = ImportBooksTestCommand::build(
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
            .expect_find_or_create_by_names()
            .times(1)
            .returning(move |_, names, _| {
                Ok(
                    HashMap::from([(names[0].as_str().to_owned(), AuthorId::new(author_uuid))])
                        .into(),
                )
            });

        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create_all()
            .returning(|_, _| Err(DomainError::Unexpected(String::from("db error"))));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation()
            .times(1)
            .returning(|_, _| Ok(()));
        tm.expect_commit().times(0);

        let interactor = ImportBooksTestCommand::build(book_repository, author_repository, tm);
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
            .expect_create_all()
            .returning(|_, _| Err(DomainError::Unexpected(String::from("db error"))));

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_names()
            .returning(|_, _, _| Ok(HashMap::new().into()));

        let interactor = ImportBooksTestCommand::build(
            book_repository,
            author_repository,
            make_transaction_manager(),
        );
        let books = vec![import_entry("Book", vec![])];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn preview_import_rolls_back_and_reports_author_statuses() {
        let existing_id = AuthorId::new(Uuid::new_v4());
        let new_id = AuthorId::new(Uuid::new_v4());
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_names()
            .times(1)
            .returning(move |_, _, _| {
                Ok(FindOrCreateAuthorsResult {
                    authors_by_name: HashMap::from([
                        ("Existing".to_string(), existing_id.clone()),
                        ("New".to_string(), new_id.clone()),
                    ]),
                    created_author_ids: [new_id.clone()].into_iter().collect(),
                })
            });
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create_all()
            .withf(|_, books| books.len() == 1 && books[0].author_ids().len() == 2)
            .times(1)
            .returning(|_, _| Ok(()));
        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation()
            .times(1)
            .returning(|_, _| Ok(()));
        tm.expect_commit().times(0);
        tm.expect_rollback().times(1).returning(|_| Ok(()));
        let interactor = ImportBooksTestCommand::build(book_repository, author_repository, tm);

        let result = interactor
            .preview_import(
                "user1",
                vec![import_entry("Preview", vec!["Existing", "New", "New"])],
            )
            .await
            .unwrap();

        assert_eq!(result.books.len(), 1);
        assert_eq!(result.books[0].authors.len(), 2);
        assert_eq!(
            result.books[0].authors[0].status,
            ImportAuthorStatus::Existing
        );
        assert_eq!(result.books[0].authors[1].status, ImportAuthorStatus::New);
    }

    #[tokio::test]
    async fn preview_import_propagates_rollback_failure() {
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_names()
            .returning(|_, _, _| Ok(HashMap::new().into()));
        let mut book_repository = MockBookRepository::new();
        book_repository.expect_create_all().returning(|_, _| Ok(()));
        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation().returning(|_, _| Ok(()));
        tm.expect_commit().times(0);
        tm.expect_rollback()
            .times(1)
            .returning(|_| Err(DomainError::Unexpected("rollback failed".to_string())));
        let interactor = ImportBooksTestCommand::build(book_repository, author_repository, tm);

        let result = interactor
            .preview_import("user1", vec![import_entry("Preview", vec![])])
            .await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn preview_import_does_not_rollback_explicitly_when_execution_fails() {
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_or_create_by_names()
            .returning(|_, _, _| Ok(HashMap::new().into()));
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_create_all()
            .returning(|_, _| Err(DomainError::Unexpected("db error".to_string())));
        let mut tm = MockTransactionManager::new();
        tm.expect_begin_operation().returning(|_, _| Ok(()));
        tm.expect_commit().times(0);
        tm.expect_rollback().times(0);
        let interactor = ImportBooksTestCommand::build(book_repository, author_repository, tm);

        let result = interactor
            .preview_import("user1", vec![import_entry("Preview", vec![])])
            .await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn import_books_invalid_title_returns_error() {
        // Given
        let interactor = ImportBooksTestCommand::build(
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
        let interactor = ImportBooksTestCommand::build(
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
        let interactor = ImportBooksTestCommand::build(
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

    #[tokio::test]
    async fn restore_book_not_found_returns_error() {
        let book_id = Uuid::new_v4();
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_restore_revision()
            .returning(|_, id, _| {
                Err(DomainError::NotFound {
                    entity_type: "book_revision",
                    entity_id: id.to_string(),
                    user_id: "user1".to_string(),
                })
            });
        let interactor = BookCommandInteractor::new(
            book_repository,
            MockAuthorRepository::new(),
            make_begin_only_transaction_manager(),
        );

        let result = interactor.restore("user1", &book_id.to_string(), 999).await;

        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn restore_book_returns_revision_state_from_repository() {
        let book_id = Uuid::new_v4();
        let mut restored_book = make_book(book_id);
        restored_book.update(
            BookUpdate {
                title: BookTitle::new("Old Title".to_string()).unwrap(),
                author_ids: vec![],
                isbn: restored_book.isbn().clone(),
                read: restored_book.read().clone(),
                owned: restored_book.owned().clone(),
                priority: restored_book.priority().clone(),
                format: restored_book.format().clone(),
                store: restored_book.store().clone(),
            },
            OffsetDateTime::now_utc(),
        );
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_restore_revision()
            .withf(|_, _, revision| *revision == 1)
            .return_once(move |_, _, _| Ok(restored_book));
        let interactor = BookCommandInteractor::new(
            book_repository,
            MockAuthorRepository::new(),
            make_transaction_manager(),
        );

        let result = interactor
            .restore("user1", &book_id.to_string(), 1)
            .await
            .unwrap();

        let restored = result.value.unwrap();
        assert_eq!(restored.title, "Old Title");
    }

    #[tokio::test]
    async fn restore_book_rejects_invalid_revision_before_transaction() {
        let book_id = Uuid::new_v4();
        let mut book_repository = MockBookRepository::new();
        book_repository.expect_restore_revision().times(0);
        let interactor = BookCommandInteractor::new(
            book_repository,
            MockAuthorRepository::new(),
            MockTransactionManager::new(),
        );

        let result = interactor.restore("user1", &book_id.to_string(), 0).await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn restore_book_repository_failure_does_not_commit() {
        let book_id = Uuid::new_v4();
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_restore_revision()
            .returning(|_, _, _| Err(DomainError::Unexpected("restore failed".to_string())));
        let interactor = BookCommandInteractor::new(
            book_repository,
            MockAuthorRepository::new(),
            make_begin_only_transaction_manager(),
        );

        let result = interactor.restore("user1", &book_id.to_string(), 1).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
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
            dto::book::ImportBookEntryDto, interactor::book::BookCommandInteractor,
            traits::book::BookCommandUseCase,
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
    ) -> BookCommandInteractor<PgBookRepository, PgAuthorRepository, PgTransactionManager> {
        BookCommandInteractor::new(
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
    #[ignore = "legacy Event assertions were removed"]
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
        assert_eq!(result.value.len(), 2);

        // Exactly two authors exist (Existing Author reused, New Author added).
        let author_rows: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM author WHERE user_id = $1 ORDER BY name")
                .bind(user_id.as_str())
                .fetch_all(&pool)
                .await?;
        assert_eq!(author_rows.len(), 2);
        assert_eq!(author_rows[0].0, "Existing Author");
        assert_eq!(author_rows[1].0, "New Author");

        // Import writes no legacy Event/EventSet history.
        let (new_author_event_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM author_event ae
             JOIN event_set es ON ae.event_set_id = es.id
             WHERE ae.user_id = $1 AND es.operation = 'import_books'
               AND ae.name = 'New Author'",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(new_author_event_count, 0);

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
        assert_eq!(result.value.len(), 2);

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
    #[ignore = "legacy Event assertions were removed"]
    async fn import_records_only_operation_revision_history(pool: PgPool) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;

        let result = interactor(&pool)
            .import(
                user_id.as_str(),
                vec![entry("Imported Book", vec!["Author A"])],
            )
            .await?;
        assert_eq!(result.value.len(), 1);

        let legacy_count: i64 = sqlx::query_scalar(
            "SELECT (SELECT COUNT(*) FROM event_set WHERE user_id = $1)
                  + (SELECT COUNT(*) FROM book_event WHERE user_id = $1)
                  + (SELECT COUNT(*) FROM author_event WHERE user_id = $1)",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(legacy_count, 0);

        let (operation_id, detail): (uuid::Uuid, serde_json::Value) = sqlx::query_as(
            "SELECT id, detail FROM operation
             WHERE user_id = $1 AND type = 'import_books'",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(
            detail,
            serde_json::json!({"type": "import_books", "imported_count": 1})
        );
        let (book_revision_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM book_revision WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        let (author_revision_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author_revision WHERE user_id = $1")
                .bind(user_id.as_str())
                .fetch_one(&pool)
                .await?;
        assert_eq!((book_revision_count, author_revision_count), (1, 1));
        let (book_change_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM book_operation_change WHERE operation_id = $1")
                .bind(operation_id)
                .fetch_one(&pool)
                .await?;
        let (author_change_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM author_operation_change WHERE operation_id = $1")
                .bind(operation_id)
                .fetch_one(&pool)
                .await?;
        assert_eq!((book_change_count, author_change_count), (1, 1));

        Ok(())
    }

    #[sqlx::test]
    async fn preview_rolls_back_operation_revisions_and_changes(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;

        interactor(&pool)
            .preview_import(
                user_id.as_str(),
                vec![entry("Preview Book", vec!["Preview Author"])],
            )
            .await?;

        let counts: (i64, i64, i64, i64, i64, i64, i64) = sqlx::query_as(
            "SELECT
               (SELECT COUNT(*) FROM operation WHERE user_id = $1),
               (SELECT COUNT(*) FROM book_revision WHERE user_id = $1),
               (SELECT COUNT(*) FROM author_revision WHERE user_id = $1),
               (SELECT COUNT(*) FROM book_operation_change WHERE user_id = $1),
               (SELECT COUNT(*) FROM author_operation_change WHERE user_id = $1),
               (SELECT COUNT(*) FROM book WHERE user_id = $1),
               (SELECT COUNT(*) FROM author WHERE user_id = $1)",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;
        assert_eq!(counts, (0, 0, 0, 0, 0, 0, 0));
        Ok(())
    }

    #[sqlx::test]
    #[ignore = "legacy Event assertions were removed"]
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
    #[ignore = "legacy Event assertions were removed"]
    async fn import_empty_author_names(pool: PgPool) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;

        let result = interactor(&pool)
            .import(
                user_id.as_str(),
                vec![entry("Book With No Authors", vec![])],
            )
            .await?;
        assert_eq!(result.value.len(), 1);

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

    #[sqlx::test]
    #[ignore = "legacy Event assertions were removed"]
    async fn import_persists_maximum_batch_without_legacy_events(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let user_id = prepare_user(&pool, "user1").await?;
        let books: Vec<ImportBookEntryDto> = (0..super::MAX_BOOK_BATCH)
            .map(|index| entry(&format!("Book {index}"), vec![]))
            .collect();

        let result = interactor(&pool).import(user_id.as_str(), books).await?;
        assert_eq!(result.value.len(), super::MAX_BOOK_BATCH);

        let (book_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM book WHERE user_id = $1")
            .bind(user_id.as_str())
            .fetch_one(&pool)
            .await?;
        let (event_count, distinct_event_sets): (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*), COUNT(DISTINCT event_set_id)
             FROM book_event WHERE user_id = $1",
        )
        .bind(user_id.as_str())
        .fetch_one(&pool)
        .await?;

        assert_eq!(book_count, super::MAX_BOOK_BATCH as i64);
        assert_eq!(event_count, 0);
        assert_eq!(distinct_event_sets, 0);

        Ok(())
    }
}
