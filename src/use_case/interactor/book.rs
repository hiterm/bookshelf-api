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
        use_case::book::{CreateBookUseCase, UpdateBookUseCase},
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

        self.book_repository.create(&user_id, &book).await?;

        Ok(book.into())
    }
}
