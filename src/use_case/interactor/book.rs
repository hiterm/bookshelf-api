use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::{AuthorId, AuthorName},
            book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            user::UserId,
        },
        error::DomainError,
        repository::{
            book_repository::BookRepository,
            import_books_repository::{ImportBookInput, ImportBooksRepository},
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

pub struct CreateBookInteractor<BR> {
    book_repository: BR,
}

impl<BR> CreateBookInteractor<BR> {
    pub fn new(book_repository: BR) -> Self {
        Self { book_repository }
    }
}

#[async_trait]
impl<BR> CreateBookUseCase for CreateBookInteractor<BR>
where
    BR: BookRepository,
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

        self.book_repository.create(&user_id, &book).await?;

        Ok(book.into())
    }
}

pub struct UpdateBookInteractor<BR> {
    book_repository: BR,
}

impl<BR> UpdateBookInteractor<BR> {
    pub fn new(book_repository: BR) -> Self {
        Self { book_repository }
    }
}

#[async_trait]
impl<BR> UpdateBookUseCase for UpdateBookInteractor<BR>
where
    BR: BookRepository,
{
    async fn update(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_data.id.as_str())?;
        let book = self.book_repository.find_by_id(&user_id, &book_id).await?;
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

        self.book_repository.update(&user_id, &book).await?;

        Ok(book.into())
    }
}

pub struct DeleteBookInteractor<BR> {
    book_repository: BR,
}

impl<BR> DeleteBookInteractor<BR> {
    pub fn new(book_repository: BR) -> Self {
        Self { book_repository }
    }
}

#[async_trait]
impl<BR> DeleteBookUseCase for DeleteBookInteractor<BR>
where
    BR: BookRepository,
{
    async fn delete(&self, user_id: &str, book_id: &str) -> Result<(), UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;

        self.book_repository.delete(&user_id, &book_id).await?;

        Ok(())
    }
}

pub struct ImportBooksInteractor<IBR> {
    import_books_repository: IBR,
}

impl<IBR> ImportBooksInteractor<IBR> {
    pub fn new(import_books_repository: IBR) -> Self {
        Self {
            import_books_repository,
        }
    }
}

#[async_trait]
impl<IBR> ImportBooksUseCase for ImportBooksInteractor<IBR>
where
    IBR: ImportBooksRepository,
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

        let user_id = UserId::new(user_id.to_string())?;
        let now = OffsetDateTime::now_utc();

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

        let books = self
            .import_books_repository
            .import(&user_id, inputs)
            .await?;

        Ok(books.into_iter().map(BookDto::from).collect())
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
            entity::book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            error::DomainError,
            repository::{
                book_repository::MockBookRepository,
                import_books_repository::MockImportBooksRepository,
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
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = CreateBookInteractor::new(book_repository);
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
        let interactor = CreateBookInteractor::new(book_repository);
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
            .expect_find_by_id()
            .with(always(), always())
            .returning(move |_, _| Ok(Some(book.clone())));
        book_repository
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = UpdateBookInteractor::new(book_repository);
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
            .expect_find_by_id()
            .with(always(), always())
            .returning(|_, _| Ok(None));

        let interactor = UpdateBookInteractor::new(book_repository);
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

        let interactor = DeleteBookInteractor::new(book_repository);

        // When
        let result = interactor.delete("user1", &book_id_str).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_book_fails_with_invalid_book_id() {
        // Given
        let book_repository = MockBookRepository::new();
        let interactor = DeleteBookInteractor::new(book_repository);

        // When
        let result = interactor.delete("user1", "not-a-valid-uuid").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn import_books_empty_list_returns_validation_error() {
        // Given
        let mock = MockImportBooksRepository::new();
        let interactor = ImportBooksInteractor::new(mock);

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
    async fn import_books_with_author_names() {
        // Given
        let book1 = make_book(Uuid::new_v4());
        let book2 = make_book(Uuid::new_v4());

        let mut mock = MockImportBooksRepository::new();
        mock.expect_import()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![book1.clone(), book2.clone()]));

        let interactor = ImportBooksInteractor::new(mock);
        let books = vec![
            ImportBookEntryDto {
                title: "Book One".to_string(),
                author_names: vec!["Author A".to_string()],
                isbn: "".to_string(),
                read: false,
                owned: false,
                priority: 50,
                format: BookFormat::Unknown,
                store: BookStore::Unknown,
            },
            ImportBookEntryDto {
                title: "Book Two".to_string(),
                author_names: vec!["Author B".to_string()],
                isbn: "".to_string(),
                read: false,
                owned: false,
                priority: 50,
                format: BookFormat::Unknown,
                store: BookStore::Unknown,
            },
        ];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(result.is_ok());
        let dtos = result.unwrap();
        assert_eq!(dtos.len(), 2);
    }

    #[tokio::test]
    async fn import_books_propagates_repository_error() {
        // Given
        let mut mock = MockImportBooksRepository::new();
        mock.expect_import()
            .with(always(), always())
            .returning(|_, _| Err(DomainError::Unexpected(String::from("db error"))));

        let interactor = ImportBooksInteractor::new(mock);
        let books = vec![ImportBookEntryDto {
            title: "Book".to_string(),
            author_names: vec![],
            isbn: "".to_string(),
            read: false,
            owned: false,
            priority: 50,
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
        }];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn import_books_invalid_title_returns_error() {
        // Given
        let mock = MockImportBooksRepository::new();
        let interactor = ImportBooksInteractor::new(mock);
        let books = vec![ImportBookEntryDto {
            title: "".to_string(),
            author_names: vec![],
            isbn: "".to_string(),
            read: false,
            owned: false,
            priority: 50,
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
        }];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn import_books_invalid_isbn_returns_error() {
        // Given
        let mock = MockImportBooksRepository::new();
        let interactor = ImportBooksInteractor::new(mock);
        let books = vec![ImportBookEntryDto {
            title: "Valid Title".to_string(),
            author_names: vec![],
            isbn: "1".to_string(),
            read: false,
            owned: false,
            priority: 50,
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
        }];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn import_books_invalid_author_name_returns_error() {
        // Given
        let mock = MockImportBooksRepository::new();
        let interactor = ImportBooksInteractor::new(mock);
        let books = vec![ImportBookEntryDto {
            title: "Valid Title".to_string(),
            author_names: vec!["".to_string()],
            isbn: "".to_string(),
            read: false,
            owned: false,
            priority: 50,
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
        }];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}
