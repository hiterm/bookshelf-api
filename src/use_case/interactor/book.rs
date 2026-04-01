use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::AuthorId,
            book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            user::UserId,
        },
        error::DomainError,
        repository::book_repository::BookRepository,
    },
    use_case::{
        dto::book::{BookDto, CreateBookDto, TimeInfo, UpdateBookDto},
        error::UseCaseError,
        traits::book::{CreateBookUseCase, DeleteBookUseCase, UpdateBookUseCase},
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
                })
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

#[cfg(test)]
mod tests {
    use mockall::predicate::always;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            entity::book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
            repository::book_repository::MockBookRepository,
        },
        use_case::{
            dto::book::{CreateBookDto, UpdateBookDto},
            error::UseCaseError,
            interactor::book::{CreateBookInteractor, DeleteBookInteractor, UpdateBookInteractor},
            traits::book::{CreateBookUseCase, DeleteBookUseCase, UpdateBookUseCase},
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
}
