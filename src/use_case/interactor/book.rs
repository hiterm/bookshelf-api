use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName},
            book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            user::UserId,
        },
        error::DomainError,
        repository::{author_repository::AuthorRepository, book_repository::BookRepository},
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

pub struct CreateBookInteractor<BR> {
    book_repository: BR,
    pool: PgPool,
}

impl<BR> CreateBookInteractor<BR> {
    pub fn new(book_repository: BR, pool: PgPool) -> Self {
        Self {
            book_repository,
            pool,
        }
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

        let mut tx = self.pool.begin().await?;
        self.book_repository
            .create(&mut tx, &user_id, &book)
            .await?;
        tx.commit().await?;

        Ok(book.into())
    }
}

pub struct UpdateBookInteractor<BR> {
    book_repository: BR,
    pool: PgPool,
}

impl<BR> UpdateBookInteractor<BR> {
    pub fn new(book_repository: BR, pool: PgPool) -> Self {
        Self {
            book_repository,
            pool,
        }
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

        let mut tx = self.pool.begin().await?;
        let book = self
            .book_repository
            .find_by_id(&mut tx, &user_id, &book_id)
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
        tx.commit().await?;

        Ok(book.into())
    }
}

pub struct DeleteBookInteractor<BR> {
    book_repository: BR,
    pool: PgPool,
}

impl<BR> DeleteBookInteractor<BR> {
    pub fn new(book_repository: BR, pool: PgPool) -> Self {
        Self {
            book_repository,
            pool,
        }
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

        let mut tx = self.pool.begin().await?;
        self.book_repository
            .delete(&mut tx, &user_id, &book_id)
            .await?;
        tx.commit().await?;

        Ok(())
    }
}

pub struct ImportBooksInteractor<BR, AR> {
    book_repository: BR,
    author_repository: AR,
    pool: PgPool,
}

impl<BR, AR> ImportBooksInteractor<BR, AR> {
    pub fn new(book_repository: BR, author_repository: AR, pool: PgPool) -> Self {
        Self {
            book_repository,
            author_repository,
            pool,
        }
    }
}

#[async_trait]
impl<BR, AR> ImportBooksUseCase for ImportBooksInteractor<BR, AR>
where
    BR: BookRepository,
    AR: AuthorRepository,
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

        let mut tx = self.pool.begin().await?;
        let mut result_books = Vec::with_capacity(books.len());

        for dto in books {
            let title = BookTitle::new(dto.title)?;
            let isbn = Isbn::new(dto.isbn)?;
            let priority = Priority::new(dto.priority)?;
            let read = ReadFlag::new(dto.read);
            let owned = OwnedFlag::new(dto.owned);
            let book_id = BookId::new(Uuid::new_v4())?;

            let mut author_ids = Vec::with_capacity(dto.author_names.len());
            for name in dto.author_names {
                let author_name = AuthorName::new(name)?;
                let author_id = AuthorId::new(Uuid::new_v4());
                let author = Author::new(author_id.clone(), author_name)?;
                self.author_repository
                    .create(&mut tx, &user_id, &author)
                    .await?;
                author_ids.push(author_id);
            }

            let book = Book::new(
                book_id, title, author_ids, isbn, read, owned, priority, dto.format, dto.store,
                now, now,
            )?;

            self.book_repository
                .create(&mut tx, &user_id, &book)
                .await?;

            result_books.push(book);
        }

        tx.commit().await?;

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
            entity::book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            error::DomainError,
            repository::{
                author_repository::MockAuthorRepository, book_repository::MockBookRepository,
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

    fn dummy_pool() -> sqlx::PgPool {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".to_string());
        sqlx::PgPool::connect_lazy(&url).unwrap()
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

        let interactor = CreateBookInteractor::new(book_repository, dummy_pool());
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
        let interactor = CreateBookInteractor::new(book_repository, dummy_pool());
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
            .with(always(), always(), always())
            .returning(move |_, _, _| Ok(Some(book.clone())));
        book_repository
            .expect_update()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = UpdateBookInteractor::new(book_repository, dummy_pool());
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
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(None));

        let interactor = UpdateBookInteractor::new(book_repository, dummy_pool());
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

        let interactor = DeleteBookInteractor::new(book_repository, dummy_pool());

        // When
        let result = interactor.delete("user1", &book_id_str).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_book_fails_with_invalid_book_id() {
        // Given
        let book_repository = MockBookRepository::new();
        let interactor = DeleteBookInteractor::new(book_repository, dummy_pool());

        // When
        let result = interactor.delete("user1", "not-a-valid-uuid").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn import_books_empty_list_returns_validation_error() {
        // Given
        let book_repo = MockBookRepository::new();
        let author_repo = MockAuthorRepository::new();
        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());

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
        // Given
        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_create()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let author_repo = MockAuthorRepository::new();

        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());
        let books = vec![
            ImportBookEntryDto {
                title: "Book".to_string(),
                author_names: vec![],
                isbn: "".to_string(),
                read: false,
                owned: false,
                priority: 50,
                format: BookFormat::Unknown,
                store: BookStore::Unknown,
            };
            super::MAX_BOOK_BATCH
        ];

        // When
        let result = interactor.import("user1", books).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn import_books_exceeds_max_batch_returns_validation_error() {
        // Given
        let book_repo = MockBookRepository::new();
        let author_repo = MockAuthorRepository::new();
        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());
        let books = vec![
            ImportBookEntryDto {
                title: "Book".to_string(),
                author_names: vec![],
                isbn: "".to_string(),
                read: false,
                owned: false,
                priority: 50,
                format: BookFormat::Unknown,
                store: BookStore::Unknown,
            };
            super::MAX_BOOK_BATCH + 1
        ];

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
        // Given
        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_create()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let mut author_repo = MockAuthorRepository::new();
        author_repo
            .expect_create()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());
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
        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_create()
            .with(always(), always(), always())
            .returning(|_, _, _| Err(DomainError::Unexpected(String::from("db error"))));

        let author_repo = MockAuthorRepository::new();

        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());
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
        let book_repo = MockBookRepository::new();
        let author_repo = MockAuthorRepository::new();
        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());
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
        let book_repo = MockBookRepository::new();
        let author_repo = MockAuthorRepository::new();
        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());
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
        let book_repo = MockBookRepository::new();
        let author_repo = MockAuthorRepository::new();
        let interactor = ImportBooksInteractor::new(book_repo, author_repo, dummy_pool());
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
