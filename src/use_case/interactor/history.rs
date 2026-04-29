use async_trait::async_trait;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName},
            book::{Book, BookId},
            user::UserId,
        },
        error::DomainError,
        repository::{
            author_history_repository::AuthorHistoryRepository,
            author_repository::AuthorRepository, book_history_repository::BookHistoryRepository,
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

        match self.book_repository.update(&user_id, &book).await {
            Ok(()) => {}
            Err(DomainError::NotFound { .. }) => {
                self.book_repository.create(&user_id, &book).await?;
            }
            Err(e) => return Err(UseCaseError::from(e)),
        }

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
                book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                change_set::ChangeSetId,
                history::{AuthorHistory, BookHistory, HistoryOperation},
            },
            repository::{
                author_history_repository::MockAuthorHistoryRepository,
                author_repository::MockAuthorRepository,
                book_history_repository::MockBookHistoryRepository,
                book_repository::MockBookRepository,
            },
        },
        use_case::{
            error::UseCaseError,
            traits::history::{
                ListAuthorHistoryUseCase, ListBookHistoryUseCase, RestoreAuthorUseCase,
                RestoreBookUseCase,
            },
        },
    };

    use super::*;

    fn make_book_history(book_id: Uuid) -> BookHistory {
        BookHistory {
            history_id: 1,
            change_set_id: ChangeSetId::from(Uuid::new_v4()),
            operation: HistoryOperation::Update,
            book_id: BookId::new(book_id).unwrap(),
            title: BookTitle::new("Old Title".to_string()).unwrap(),
            author_ids: vec![],
            isbn: Isbn::new("".to_string()).unwrap(),
            read: ReadFlag::new(false),
            owned: OwnedFlag::new(false),
            priority: Priority::new(50).unwrap(),
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
            book_created_at: OffsetDateTime::now_utc(),
            book_updated_at: OffsetDateTime::now_utc(),
            changed_at: OffsetDateTime::now_utc(),
        }
    }

    fn make_author_history(author_id: Uuid) -> AuthorHistory {
        AuthorHistory {
            history_id: 2,
            change_set_id: ChangeSetId::from(Uuid::new_v4()),
            operation: HistoryOperation::Update,
            author_id: AuthorId::new(author_id),
            name: "Old Name".to_string(),
            yomi: "".to_string(),
            author_created_at: OffsetDateTime::now_utc(),
            author_updated_at: OffsetDateTime::now_utc(),
            changed_at: OffsetDateTime::now_utc(),
        }
    }

    #[tokio::test]
    async fn list_book_history_returns_dto_list() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();
        let history = make_book_history(book_uuid);

        let mut repo = MockBookHistoryRepository::new();
        repo.expect_find_by_book()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![history.clone()]));

        let interactor = ListBookHistoryInteractor::new(repo);
        let result = interactor.list("user1", &book_id_str).await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Old Title");
    }

    #[tokio::test]
    async fn list_book_history_returns_empty_when_none() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();

        let mut repo = MockBookHistoryRepository::new();
        repo.expect_find_by_book()
            .with(always(), always())
            .returning(|_, _| Ok(vec![]));

        let interactor = ListBookHistoryInteractor::new(repo);
        let result = interactor.list("user1", &book_id_str).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_author_history_returns_dto_list() {
        let author_uuid = Uuid::new_v4();
        let author_id_str = author_uuid.hyphenated().to_string();
        let history = make_author_history(author_uuid);

        let mut repo = MockAuthorHistoryRepository::new();
        repo.expect_find_by_author()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![history.clone()]));

        let interactor = ListAuthorHistoryInteractor::new(repo);
        let result = interactor.list("user1", &author_id_str).await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Old Name");
    }

    #[tokio::test]
    async fn restore_book_not_found_returns_error() {
        let mut repo = MockBookHistoryRepository::new();
        repo.expect_find_by_history_id()
            .with(always(), always())
            .returning(|_, _| Ok(None));

        let book_repo = MockBookRepository::new();
        let interactor = RestoreBookInteractor::new(book_repo, repo);
        let result = interactor.restore("user1", 999).await;

        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn restore_book_success() {
        let book_uuid = Uuid::new_v4();
        let history = make_book_history(book_uuid);

        let mut history_repo = MockBookHistoryRepository::new();
        history_repo
            .expect_find_by_history_id()
            .with(always(), always())
            .returning(move |_, _| Ok(Some(history.clone())));

        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreBookInteractor::new(book_repo, history_repo);
        let result = interactor.restore("user1", 1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Old Title");
    }

    #[tokio::test]
    async fn restore_book_falls_back_to_create_when_deleted() {
        let book_uuid = Uuid::new_v4();
        let history = make_book_history(book_uuid);

        let mut history_repo = MockBookHistoryRepository::new();
        history_repo
            .expect_find_by_history_id()
            .with(always(), always())
            .returning(move |_, _| Ok(Some(history.clone())));

        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_update()
            .with(always(), always())
            .returning(|_, _| {
                Err(crate::domain::error::DomainError::NotFound {
                    entity_type: "book",
                    entity_id: "some-id".to_string(),
                    user_id: "user1".to_string(),
                })
            });
        book_repo
            .expect_create()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreBookInteractor::new(book_repo, history_repo);
        let result = interactor.restore("user1", 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn restore_author_not_found_returns_error() {
        let mut history_repo = MockAuthorHistoryRepository::new();
        history_repo
            .expect_find_by_history_id()
            .with(always(), always())
            .returning(|_, _| Ok(None));

        let author_repo = MockAuthorRepository::new();
        let interactor = RestoreAuthorInteractor::new(author_repo, history_repo);
        let result = interactor.restore("user1", 999).await;

        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn restore_author_success() {
        let author_uuid = Uuid::new_v4();
        let history = make_author_history(author_uuid);

        let mut history_repo = MockAuthorHistoryRepository::new();
        history_repo
            .expect_find_by_history_id()
            .with(always(), always())
            .returning(move |_, _| Ok(Some(history.clone())));

        let mut author_repo = MockAuthorRepository::new();
        author_repo
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreAuthorInteractor::new(author_repo, history_repo);
        let result = interactor.restore("user1", 2).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Old Name");
    }
}
