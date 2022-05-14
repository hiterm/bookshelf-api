use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{book::Book, user::UserId},
        repository::book_repository::BookRepository,
    },
    use_case::{
        dto::book::{BookDto, CreateBookDto},
        error::UseCaseError,
        use_case::book::CreateBookUseCase,
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
        let book = Book::try_from((uuid, book_data))?;

        self.book_repository.create(&user_id, &book).await?;

        Ok(book.into())
    }
}
