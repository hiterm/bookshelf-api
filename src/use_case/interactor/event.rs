use async_trait::async_trait;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName},
            book::{Book, BookId},
            event::{EventOperation, EventSetOperation},
            user::UserId,
        },
        repository::{
            author_event_repository::AuthorEventRepository,
            author_repository::AuthorRepository,
            book_event_repository::BookEventRepository,
            book_repository::BookRepository,
            transaction::{TransactionEventSet, TransactionManager},
        },
    },
    use_case::{
        dto::{
            author::AuthorDto,
            book::BookDto,
            event::{AuthorEventDto, BookEventDto},
            mutation::{MutationResultDto, RestoreAuthorResultDto, RestoreBookResultDto},
        },
        error::UseCaseError,
        traits::event::{
            ListAuthorEventsUseCase, ListBookEventsUseCase, RestoreAuthorUseCase,
            RestoreBookUseCase,
        },
    },
};

pub struct ListBookEventsInteractor<BER> {
    book_event_repository: BER,
}

impl<BER> ListBookEventsInteractor<BER> {
    pub fn new(book_event_repository: BER) -> Self {
        Self {
            book_event_repository,
        }
    }
}

#[async_trait]
impl<BER> ListBookEventsUseCase for ListBookEventsInteractor<BER>
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

pub struct ListAuthorEventsInteractor<AER> {
    author_event_repository: AER,
}

impl<AER> ListAuthorEventsInteractor<AER> {
    pub fn new(author_event_repository: AER) -> Self {
        Self {
            author_event_repository,
        }
    }
}

#[async_trait]
impl<AER> ListAuthorEventsUseCase for ListAuthorEventsInteractor<AER>
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

pub struct RestoreBookInteractor<BR, BER, TM> {
    book_repository: BR,
    book_event_repository: BER,
    transaction_manager: TM,
}

impl<BR, BER, TM> RestoreBookInteractor<BR, BER, TM> {
    pub fn new(book_repository: BR, book_event_repository: BER, transaction_manager: TM) -> Self {
        Self {
            book_repository,
            book_event_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<BR, BER, TM> RestoreBookUseCase for RestoreBookInteractor<BR, BER, TM>
where
    TM: TransactionManager,
    BR: BookRepository<Transaction = TM::Transaction>,
    BER: BookEventRepository,
{
    async fn restore(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreBookResultDto, UseCaseError> {
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
            EventOperation::Create
            | EventOperation::Update
            | EventOperation::Restore
            | EventOperation::Snapshot => {
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

                let dto = BookDto::from(book.clone());
                let mut tx = self
                    .transaction_manager
                    .begin(&user_id, EventSetOperation::RestoreBook)
                    .await?;
                self.book_repository
                    .restore(&mut tx, event_id, Some(book))
                    .await?;
                let event_set_id = tx.event_set_id().hyphenated().to_string();
                self.transaction_manager.commit(tx).await?;
                Ok(MutationResultDto::new(Some(dto), event_set_id))
            }
            EventOperation::Delete => {
                let mut tx = self
                    .transaction_manager
                    .begin(&user_id, EventSetOperation::RestoreBook)
                    .await?;
                self.book_repository
                    .restore(&mut tx, event_id, None)
                    .await?;
                let event_set_id = tx.event_set_id().hyphenated().to_string();
                self.transaction_manager.commit(tx).await?;
                Ok(MutationResultDto::new(None, event_set_id))
            }
        }
    }
}

pub struct RestoreAuthorInteractor<AR, AER, TM> {
    author_repository: AR,
    author_event_repository: AER,
    transaction_manager: TM,
}

impl<AR, AER, TM> RestoreAuthorInteractor<AR, AER, TM> {
    pub fn new(
        author_repository: AR,
        author_event_repository: AER,
        transaction_manager: TM,
    ) -> Self {
        Self {
            author_repository,
            author_event_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<AR, AER, TM> RestoreAuthorUseCase for RestoreAuthorInteractor<AR, AER, TM>
where
    TM: TransactionManager,
    AR: AuthorRepository<Transaction = TM::Transaction>,
    AER: AuthorEventRepository,
{
    async fn restore(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreAuthorResultDto, UseCaseError> {
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
            EventOperation::Create
            | EventOperation::Update
            | EventOperation::Restore
            | EventOperation::Snapshot => {
                let name = event.name.ok_or_else(|| {
                    UseCaseError::Validation("author_event name is null".to_string())
                })?;
                let yomi = event.yomi.ok_or_else(|| {
                    UseCaseError::Validation("author_event yomi is null".to_string())
                })?;
                let author_name = AuthorName::new(name)?;
                let created_at = event.author_created_at.ok_or_else(|| {
                    UseCaseError::Validation("author_event author_created_at is null".to_string())
                })?;
                let updated_at = event.author_updated_at.ok_or_else(|| {
                    UseCaseError::Validation("author_event author_updated_at is null".to_string())
                })?;
                let author = Author::new_with_timestamps(
                    event.author_id,
                    author_name,
                    yomi,
                    created_at,
                    updated_at,
                )?;

                let dto = AuthorDto::from(author.clone());
                let mut tx = self
                    .transaction_manager
                    .begin(&user_id, EventSetOperation::RestoreAuthor)
                    .await?;
                self.author_repository
                    .restore(&mut tx, event_id, Some(author))
                    .await?;
                let event_set_id = tx.event_set_id().hyphenated().to_string();
                self.transaction_manager.commit(tx).await?;
                Ok(MutationResultDto::new(Some(dto), event_set_id))
            }
            EventOperation::Delete => {
                let mut tx = self
                    .transaction_manager
                    .begin(&user_id, EventSetOperation::RestoreAuthor)
                    .await?;
                self.author_repository
                    .restore(&mut tx, event_id, None)
                    .await?;
                let event_set_id = tx.event_set_id().hyphenated().to_string();
                self.transaction_manager.commit(tx).await?;
                Ok(MutationResultDto::new(None, event_set_id))
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
                event::{AuthorEvent, BookEvent, EventOperation},
                event_set::EventSetId,
            },
            repository::{
                author_event_repository::MockAuthorEventRepository,
                author_repository::MockAuthorRepository,
                book_event_repository::MockBookEventRepository,
                book_repository::MockBookRepository, transaction::MockTransactionManager,
            },
        },
        use_case::{
            error::UseCaseError,
            traits::event::{
                ListAuthorEventsUseCase, ListBookEventsUseCase, RestoreAuthorUseCase,
                RestoreBookUseCase,
            },
        },
    };

    use super::*;

    // A MockTransactionManager whose Transaction associated type is () and
    // whose begin/commit succeed, for restore paths that reach the repository.
    fn make_transaction_manager() -> MockTransactionManager {
        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_commit().returning(|_| Ok(()));
        tm
    }

    fn make_book_event(book_id: Uuid) -> BookEvent {
        BookEvent {
            event_id: 1,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: EventOperation::Update,
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
            extra: None,
        }
    }

    fn make_book_delete_event(book_id: Uuid) -> BookEvent {
        BookEvent {
            event_id: 10,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: EventOperation::Delete,
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
            extra: None,
        }
    }

    fn make_author_event(author_id: Uuid) -> AuthorEvent {
        AuthorEvent {
            event_id: 2,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: EventOperation::Update,
            author_id: AuthorId::new(author_id),
            name: Some("Old Name".to_string()),
            yomi: Some("おーるど".to_string()),
            author_created_at: Some(OffsetDateTime::now_utc()),
            author_updated_at: Some(OffsetDateTime::now_utc()),
            changed_at: OffsetDateTime::now_utc(),
            extra: None,
        }
    }

    fn make_author_delete_event(author_id: Uuid) -> AuthorEvent {
        AuthorEvent {
            event_id: 20,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: EventOperation::Delete,
            author_id: AuthorId::new(author_id),
            name: None,
            yomi: None,
            author_created_at: None,
            author_updated_at: None,
            changed_at: OffsetDateTime::now_utc(),
            extra: None,
        }
    }

    #[tokio::test]
    async fn list_book_events_returns_dto_list() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();
        let event = make_book_event(book_uuid);

        let mut repo = MockBookEventRepository::new();
        repo.expect_find_by_book()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![event.clone()]));

        let interactor = ListBookEventsInteractor::new(repo);
        let result = interactor.list("user1", &book_id_str).await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title.as_deref(), Some("Old Title"));
    }

    #[tokio::test]
    async fn list_book_events_returns_empty_when_none() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();

        let mut repo = MockBookEventRepository::new();
        repo.expect_find_by_book()
            .with(always(), always())
            .returning(|_, _| Ok(vec![]));

        let interactor = ListBookEventsInteractor::new(repo);
        let result = interactor.list("user1", &book_id_str).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_author_events_returns_dto_list() {
        let author_uuid = Uuid::new_v4();
        let author_id_str = author_uuid.hyphenated().to_string();
        let event = make_author_event(author_uuid);

        let mut repo = MockAuthorEventRepository::new();
        repo.expect_find_by_author()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![event.clone()]));

        let interactor = ListAuthorEventsInteractor::new(repo);
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
        let interactor = RestoreBookInteractor::new(book_repo, repo, MockTransactionManager::new());
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
            .expect_restore()
            .with(always(), eq(1i64), always())
            .returning(|_, _, _| Ok(()));

        let interactor =
            RestoreBookInteractor::new(book_repo, history_repo, make_transaction_manager());
        let result = interactor.restore("user1", 1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().value.unwrap().title, "Old Title");
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
            .expect_restore()
            .with(always(), eq(10i64), always())
            .returning(|_, _, _| Ok(()));

        let interactor =
            RestoreBookInteractor::new(book_repo, history_repo, make_transaction_manager());
        let result = interactor.restore("user1", 10).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn restore_book_snapshot_event_applies_state() {
        let book_uuid = Uuid::new_v4();
        let mut event = make_book_event(book_uuid);
        event.operation = EventOperation::Snapshot;

        let mut history_repo = MockBookEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(1i64))
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut book_repo = MockBookRepository::new();
        book_repo
            .expect_restore()
            .with(always(), eq(1i64), always())
            .returning(|_, _, _| Ok(()));

        let interactor =
            RestoreBookInteractor::new(book_repo, history_repo, make_transaction_manager());
        let result = interactor.restore("user1", 1).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn restore_author_not_found_returns_error() {
        let mut history_repo = MockAuthorEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(999i64))
            .returning(|_, _| Ok(None));

        let author_repo = MockAuthorRepository::new();
        let interactor =
            RestoreAuthorInteractor::new(author_repo, history_repo, MockTransactionManager::new());
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
            .expect_restore()
            .withf(|_, event_id, author| {
                *event_id == 2
                    && author
                        .as_ref()
                        .is_some_and(|author| author.yomi() == "おーるど")
            })
            .returning(|_, _, _| Ok(()));

        let interactor =
            RestoreAuthorInteractor::new(author_repo, history_repo, make_transaction_manager());
        let result = interactor.restore("user1", 2).await;

        assert!(result.is_ok());
        let restored = result.unwrap().value.unwrap();
        assert_eq!(restored.name, "Old Name");
        assert_eq!(restored.yomi, "おーるど");
    }

    #[tokio::test]
    async fn restore_author_preserves_legacy_yomi() {
        let author_uuid = Uuid::new_v4();
        let mut event = make_author_event(author_uuid);
        event.yomi = Some("authora".to_string());

        let mut history_repo = MockAuthorEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(2i64))
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut author_repo = MockAuthorRepository::new();
        author_repo
            .expect_restore()
            .withf(|_, event_id, author| {
                *event_id == 2
                    && author
                        .as_ref()
                        .is_some_and(|author| author.yomi() == "authora")
            })
            .returning(|_, _, _| Ok(()));

        let interactor =
            RestoreAuthorInteractor::new(author_repo, history_repo, make_transaction_manager());
        let result = interactor.restore("user1", 2).await;

        assert_eq!(result.unwrap().value.unwrap().yomi, "authora");
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
            .expect_restore()
            .with(always(), eq(20i64), always())
            .returning(|_, _, _| Ok(()));

        let interactor =
            RestoreAuthorInteractor::new(author_repo, history_repo, make_transaction_manager());
        let result = interactor.restore("user1", 20).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn restore_author_snapshot_event_applies_state() {
        let author_uuid = Uuid::new_v4();
        let mut event = make_author_event(author_uuid);
        event.operation = EventOperation::Snapshot;

        let mut history_repo = MockAuthorEventRepository::new();
        history_repo
            .expect_find_by_event_id()
            .with(always(), eq(2i64))
            .returning(move |_, _| Ok(Some(event.clone())));

        let mut author_repo = MockAuthorRepository::new();
        author_repo
            .expect_restore()
            .with(always(), eq(2i64), always())
            .returning(|_, _, _| Ok(()));

        let interactor =
            RestoreAuthorInteractor::new(author_repo, history_repo, make_transaction_manager());
        let result = interactor.restore("user1", 2).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
