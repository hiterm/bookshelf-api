use async_trait::async_trait;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName},
            book::{Book, BookId},
            user::UserId,
        },
        repository::{
            author_history_repository::AuthorHistoryRepository,
            author_repository::AuthorRepository,
            book_history_repository::BookHistoryRepository,
            book_repository::BookRepository,
        },
    },
    use_case::{
        dto::{
            author::AuthorDto,
            book::BookDto,
            history::{AuthorHistoryDto, BookHistoryDto},
        },
        error::UseCaseError,
        traits::history::{
            ListAuthorHistoryUseCase, ListBookHistoryUseCase, RestoreAuthorUseCase,
            RestoreBookUseCase,
        },
    },
};

pub struct ListBookHistoryInteractor<BHR> {
    book_history_repository: BHR,
}

impl<BHR> ListBookHistoryInteractor<BHR> {
    pub fn new(book_history_repository: BHR) -> Self {
        Self {
            book_history_repository,
        }
    }
}

#[async_trait]
impl<BHR> ListBookHistoryUseCase for ListBookHistoryInteractor<BHR>
where
    BHR: BookHistoryRepository,
{
    async fn list(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Vec<BookHistoryDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        let entries = self
            .book_history_repository
            .find_by_book(&user_id, &book_id)
            .await?;
        Ok(entries.into_iter().map(BookHistoryDto::from).collect())
    }
}

pub struct ListAuthorHistoryInteractor<AHR> {
    author_history_repository: AHR,
}

impl<AHR> ListAuthorHistoryInteractor<AHR> {
    pub fn new(author_history_repository: AHR) -> Self {
        Self {
            author_history_repository,
        }
    }
}

#[async_trait]
impl<AHR> ListAuthorHistoryUseCase for ListAuthorHistoryInteractor<AHR>
where
    AHR: AuthorHistoryRepository,
{
    async fn list(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorHistoryDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;
        let entries = self
            .author_history_repository
            .find_by_author(&user_id, &author_id)
            .await?;
        Ok(entries.into_iter().map(AuthorHistoryDto::from).collect())
    }
}

pub struct RestoreBookInteractor<BR, BHR> {
    book_repository: BR,
    book_history_repository: BHR,
}

impl<BR, BHR> RestoreBookInteractor<BR, BHR> {
    pub fn new(book_repository: BR, book_history_repository: BHR) -> Self {
        Self {
            book_repository,
            book_history_repository,
        }
    }
}

#[async_trait]
impl<BR, BHR> RestoreBookUseCase for RestoreBookInteractor<BR, BHR>
where
    BR: BookRepository,
    BHR: BookHistoryRepository,
{
    async fn restore(&self, user_id: &str, history_id: i64) -> Result<BookDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let snapshot = self
            .book_history_repository
            .find_by_history_id(&user_id, history_id)
            .await?
            .ok_or(UseCaseError::NotFound {
                entity_type: "book_history",
                entity_id: history_id.to_string(),
                user_id: user_id.as_str().to_string(),
            })?;

        let book = Book::new(
            snapshot.book_id,
            snapshot.title,
            snapshot.author_ids,
            snapshot.isbn,
            snapshot.read,
            snapshot.owned,
            snapshot.priority,
            snapshot.format,
            snapshot.store,
            snapshot.book_created_at,
            snapshot.book_updated_at,
        )?;

        self.book_repository.update(&user_id, &book).await?;

        Ok(BookDto::from(book))
    }
}

pub struct RestoreAuthorInteractor<AR, AHR> {
    author_repository: AR,
    author_history_repository: AHR,
}

impl<AR, AHR> RestoreAuthorInteractor<AR, AHR> {
    pub fn new(author_repository: AR, author_history_repository: AHR) -> Self {
        Self {
            author_repository,
            author_history_repository,
        }
    }
}

#[async_trait]
impl<AR, AHR> RestoreAuthorUseCase for RestoreAuthorInteractor<AR, AHR>
where
    AR: AuthorRepository,
    AHR: AuthorHistoryRepository,
{
    async fn restore(&self, user_id: &str, history_id: i64) -> Result<AuthorDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let snapshot = self
            .author_history_repository
            .find_by_history_id(&user_id, history_id)
            .await?
            .ok_or(UseCaseError::NotFound {
                entity_type: "author_history",
                entity_id: history_id.to_string(),
                user_id: user_id.as_str().to_string(),
            })?;

        let author_name = AuthorName::new(snapshot.name.clone())?;
        let author = Author::new(snapshot.author_id, author_name)?;
        self.author_repository.update(&user_id, &author).await?;

        Ok(AuthorDto::from(author))
    }
}
