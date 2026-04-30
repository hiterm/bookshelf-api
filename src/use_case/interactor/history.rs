use async_trait::async_trait;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName},
            book::{Book, BookId},
            history::HistoryOperation,
            user::UserId,
        },
        error::DomainError,
        repository::{
            author_event_repository::AuthorEventRepository, author_repository::AuthorRepository,
            book_event_repository::BookEventRepository, book_repository::BookRepository,
        },
    },
    use_case::{
        dto::{
            author::AuthorDto,
            book::BookDto,
            history::{AuthorEventDto, BookEventDto},
        },
        error::UseCaseError,
        traits::history::{
            ListAuthorHistoryUseCase, ListBookHistoryUseCase, RestoreAuthorUseCase,
            RestoreBookUseCase,
        },
    },
};

pub struct ListBookHistoryInteractor<BER> {
    book_event_repository: BER,
}

impl<BER> ListBookHistoryInteractor<BER> {
    pub fn new(book_event_repository: BER) -> Self {
        Self {
            book_event_repository,
        }
    }
}

#[async_trait]
impl<BER> ListBookHistoryUseCase for ListBookHistoryInteractor<BER>
where
    BER: BookEventRepository,
{
    async fn list(&self, user_id: &str, book_id: &str) -> Result<Vec<BookEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        let entries = self
            .book_event_repository
            .find_by_book(&user_id, &book_id)
            .await?;
        Ok(entries.into_iter().map(BookEventDto::from).collect())
    }
}

pub struct ListAuthorHistoryInteractor<AER> {
    author_event_repository: AER,
}

impl<AER> ListAuthorHistoryInteractor<AER> {
    pub fn new(author_event_repository: AER) -> Self {
        Self {
            author_event_repository,
        }
    }
}

#[async_trait]
impl<AER> ListAuthorHistoryUseCase for ListAuthorHistoryInteractor<AER>
where
    AER: AuthorEventRepository,
{
    async fn list(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;
        let entries = self
            .author_event_repository
            .find_by_author(&user_id, &author_id)
            .await?;
        Ok(entries.into_iter().map(AuthorEventDto::from).collect())
    }
}

pub struct RestoreBookInteractor<BR, BER> {
    book_repository: BR,
    book_event_repository: BER,
}

impl<BR, BER> RestoreBookInteractor<BR, BER> {
    pub fn new(book_repository: BR, book_event_repository: BER) -> Self {
        Self {
            book_repository,
            book_event_repository,
        }
    }
}

#[async_trait]
impl<BR, BER> RestoreBookUseCase for RestoreBookInteractor<BR, BER>
where
    BR: BookRepository,
    BER: BookEventRepository,
{
    async fn restore(&self, user_id: &str, event_id: i64) -> Result<Option<BookDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let event = self
            .book_event_repository
            .find_by_event_id(&user_id, event_id)
            .await?
            .ok_or(UseCaseError::NotFound {
                entity_type: "book_event",
                entity_id: event_id.to_string(),
                user_id: user_id.as_str().to_string(),
            })?;

        match event.operation {
            HistoryOperation::Create | HistoryOperation::Update => {
                let book = Book::new(
                    event.book_id,
                    event.title.ok_or_else(|| {
                        UseCaseError::Validation("book_event title is null".to_string())
                    })?,
                    event.author_ids,
                    event.isbn.ok_or_else(|| {
                        UseCaseError::Validation("book_event isbn is null".to_string())
                    })?,
                    event.read.ok_or_else(|| {
                        UseCaseError::Validation("book_event read is null".to_string())
                    })?,
                    event.owned.ok_or_else(|| {
                        UseCaseError::Validation("book_event owned is null".to_string())
                    })?,
                    event.priority.ok_or_else(|| {
                        UseCaseError::Validation("book_event priority is null".to_string())
                    })?,
                    event.format.ok_or_else(|| {
                        UseCaseError::Validation("book_event format is null".to_string())
                    })?,
                    event.store.ok_or_else(|| {
                        UseCaseError::Validation("book_event store is null".to_string())
                    })?,
                    event.book_created_at.ok_or_else(|| {
                        UseCaseError::Validation("book_event book_created_at is null".to_string())
                    })?,
                    event.book_updated_at.ok_or_else(|| {
                        UseCaseError::Validation("book_event book_updated_at is null".to_string())
                    })?,
                )?;

                match self.book_repository.update(&user_id, &book).await {
                    Ok(()) => {}
                    Err(DomainError::NotFound { .. }) => {
                        self.book_repository.create(&user_id, &book).await?;
                    }
                    Err(e) => return Err(UseCaseError::from(e)),
                }

                Ok(Some(BookDto::from(book)))
            }
            HistoryOperation::Delete => {
                match self.book_repository.delete(&user_id, &event.book_id).await {
                    Ok(()) | Err(DomainError::NotFound { .. }) => {}
                    Err(e) => return Err(UseCaseError::from(e)),
                }
                Ok(None)
            }
        }
    }
}

pub struct RestoreAuthorInteractor<AR, AER> {
    author_repository: AR,
    author_event_repository: AER,
}

impl<AR, AER> RestoreAuthorInteractor<AR, AER> {
    pub fn new(author_repository: AR, author_event_repository: AER) -> Self {
        Self {
            author_repository,
            author_event_repository,
        }
    }
}

#[async_trait]
impl<AR, AER> RestoreAuthorUseCase for RestoreAuthorInteractor<AR, AER>
where
    AR: AuthorRepository,
    AER: AuthorEventRepository,
{
    async fn restore(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<Option<AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let event = self
            .author_event_repository
            .find_by_event_id(&user_id, event_id)
            .await?
            .ok_or(UseCaseError::NotFound {
                entity_type: "author_event",
                entity_id: event_id.to_string(),
                user_id: user_id.as_str().to_string(),
            })?;

        match event.operation {
            HistoryOperation::Create | HistoryOperation::Update => {
                let name = event.name.ok_or_else(|| {
                    UseCaseError::Validation("author_event name is null".to_string())
                })?;
                let author_name = AuthorName::new(name)?;
                let author = Author::new(event.author_id, author_name)?;

                match self.author_repository.update(&user_id, &author).await {
                    Ok(()) => {}
                    Err(DomainError::NotFound { .. }) => {
                        self.author_repository.create(&user_id, &author).await?;
                    }
                    Err(e) => return Err(UseCaseError::from(e)),
                }

                Ok(Some(AuthorDto::from(author)))
            }
            HistoryOperation::Delete => {
                match self
                    .author_repository
                    .delete(&user_id, &event.author_id)
                    .await
                {
                    Ok(()) | Err(DomainError::NotFound { .. }) => {}
                    Err(e) => return Err(UseCaseError::from(e)),
                }
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::{always, eq};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            entity::{
                author::AuthorId,
                book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                event_set::EventSetId,
                history::{AuthorEvent, BookEvent, HistoryOperation},
            },
            repository::{
                author_event_repository::MockAuthorEventRepository,
                author_repository::MockAuthorRepository,
                book_event_repository::MockBookEventRepository,
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

    fn make_book_event(book_id: Uuid) -> BookEvent {
        BookEvent {
            event_id: 1,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: HistoryOperation::Update,
            book_id: BookId::new(book_id).unwrap(),
            title: Some(BookTitle::new("Old Title".to_string()).unwrap()),
            author_ids: vec![],
            isbn: Some(Isbn::new("".to_string()).unwrap()),
            read: Some(ReadFlag::new(false)),
            owned: Some(OwnedFlag::new(false)),
            priority: Some(Priority::new(50).unwrap()),
            format: Some(BookFormat::Unknown),
            store: Some(BookStore::Unknown),
            book_created_at: Some(OffsetDateTime::now_utc()),
            book_updated_at: Some(OffsetDateTime::now_utc()),
            changed_at: OffsetDateTime::now_utc(),
        }
    }

    fn make_book_delete_event(book_id: Uuid) -> BookEvent {
        BookEvent {
            event_id: 10,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: HistoryOperation::Delete,
            book_id: BookId::new(book_id).unwrap(),
            title: None,
            author_ids: vec![],
            isbn: None,
            read: None,
            owned: None,
            priority: None,
            format: None,
            store: None,
            book_created_at: None,
            book_updated_at: None,
            changed_at: OffsetDateTime::now_utc(),
        }
    }

    fn make_author_event(author_id: Uuid) -> AuthorEvent {
        AuthorEvent {
            event_id: 2,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: HistoryOperation::Update,
            author_id: AuthorId::new(author_id),
            name: Some("Old Name".to_string()),
            yomi: Some("".to_string()),
            author_created_at: Some(OffsetDateTime::now_utc()),
            author_updated_at: Some(OffsetDateTime::now_utc()),
            changed_at: OffsetDateTime::now_utc(),
        }
    }

    fn make_author_delete_event(author_id: Uuid) -> AuthorEvent {
        AuthorEvent {
            event_id: 20,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: HistoryOperation::Delete,
            author_id: AuthorId::new(author_id),
            name: None,
            yomi: None,
            author_created_at: None,
            author_updated_at: None,
            changed_at: OffsetDateTime::now_utc(),
        }
    }

    #[tokio::test]
    async fn list_book_history_returns_dto_list() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();
        let event = make_book_event(book_uuid);

        let mut repo = MockBookEventRepository::new();
        repo.expect_find_by_book()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![event.clone()]));

        let interactor = ListBookHistoryInteractor::new(repo);
        let result = interactor.list("user1", &book_id_str).await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title.as_deref(), Some("Old Title"));
    }

    #[tokio::test]
    async fn list_book_history_returns_empty_when_none() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();

        let mut repo = MockBookEventRepository::new();
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
        let event = make_author_event(author_uuid);

        let mut repo = MockAuthorEventRepository::new();
        repo.expect_find_by_author()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![event.clone()]));

        let interactor = ListAuthorHistoryInteractor::new(repo);
        let result = interactor.list("user1", &author_id_str).await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name.as_deref(), Some("Old Name"));
    }

    #[tokio::test]
    async fn restore_book_not_found_returns_error() {
        let mut repo = MockBookEventRepository::new();
        repo.expect_find_by_event_id()
            .with(always(), eq(999i64))
            .returning(|_, _| Ok(None));

        let book_repo = MockBookRepository::new();
        let interactor = RestoreBookInteractor::new(book_repo, repo);
        let result = interactor.restore("user1", 999).await;

        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn restore_book_success() {
        let book_uuid = Uuid::new_v4();
        let event = make_book_event(book_uuid);

        let mut history_repo = MockBookEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(1i64))
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreBookInteractor::new(book_repo, history_repo);
        let result = interactor.restore("user1", 1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap().title, "Old Title");
    }

    #[tokio::test]
    async fn restore_book_falls_back_to_create_when_deleted() {
        let book_uuid = Uuid::new_v4();
        let event = make_book_event(book_uuid);

        let mut history_repo = MockBookEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), always())
            .returning(move |_, _| Ok(Some(event.clone())));

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
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn restore_book_delete_event_deletes_book() {
        let book_uuid = Uuid::new_v4();
        let event = make_book_delete_event(book_uuid);

        let mut history_repo = MockBookEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(10i64))
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreBookInteractor::new(book_repo, history_repo);
        let result = interactor.restore("user1", 10).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn restore_author_not_found_returns_error() {
        let mut history_repo = MockAuthorEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(999i64))
            .returning(|_, _| Ok(None));

        let author_repo = MockAuthorRepository::new();
        let interactor = RestoreAuthorInteractor::new(author_repo, history_repo);
        let result = interactor.restore("user1", 999).await;

        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn restore_author_success() {
        let author_uuid = Uuid::new_v4();
        let event = make_author_event(author_uuid);

        let mut history_repo = MockAuthorEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(2i64))
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut author_repo = MockAuthorRepository::new();
        author_repo
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreAuthorInteractor::new(author_repo, history_repo);
        let result = interactor.restore("user1", 2).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap().name, "Old Name");
    }

    #[tokio::test]
    async fn restore_author_falls_back_to_create_when_deleted() {
        let author_uuid = Uuid::new_v4();
        let event = make_author_event(author_uuid);

        let mut history_repo = MockAuthorEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), always())
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut author_repo = MockAuthorRepository::new();
        author_repo
            .expect_update()
            .with(always(), always())
            .returning(|_, _| {
                Err(crate::domain::error::DomainError::NotFound {
                    entity_type: "author",
                    entity_id: "some-id".to_string(),
                    user_id: "user1".to_string(),
                })
            });
        author_repo
            .expect_create()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreAuthorInteractor::new(author_repo, history_repo);
        let result = interactor.restore("user1", 2).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn restore_author_delete_event_deletes_author() {
        let author_uuid = Uuid::new_v4();
        let event = make_author_delete_event(author_uuid);

        let mut history_repo = MockAuthorEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(20i64))
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut author_repo = MockAuthorRepository::new();
        author_repo
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = RestoreAuthorInteractor::new(author_repo, history_repo);
        let result = interactor.restore("user1", 20).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
