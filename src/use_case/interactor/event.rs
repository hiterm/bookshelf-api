use crate::{
    domain::{
        entity::{author::AuthorId, book::BookId, event_set::EventSetId, user::UserId},
        repository::{
            author_event_repository::AuthorEventRepository,
            book_event_repository::BookEventRepository, event_set_repository::EventSetRepository,
        },
    },
    use_case::{
        dto::{
            event::{AuthorEventDto, BookEventDto},
            event_set::EventSetDto,
        },
        error::UseCaseError,
        traits::event::EventQueryUseCase,
    },
};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct EventQueryInteractor<BER, AER, ESR> {
    book_event_repository: BER,
    author_event_repository: AER,
    event_set_repository: ESR,
}

impl<BER, AER, ESR> EventQueryInteractor<BER, AER, ESR> {
    pub fn new(
        book_event_repository: BER,
        author_event_repository: AER,
        event_set_repository: ESR,
    ) -> Self {
        Self {
            book_event_repository,
            author_event_repository,
            event_set_repository,
        }
    }
}

#[async_trait]
impl<BER, AER, ESR> EventQueryUseCase for EventQueryInteractor<BER, AER, ESR>
where
    BER: BookEventRepository,
    AER: AuthorEventRepository,
    ESR: EventSetRepository,
{
    async fn list_book_events(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Vec<BookEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        Ok(self
            .book_event_repository
            .find_by_book(&user_id, &book_id)
            .await?
            .into_iter()
            .map(BookEventDto::from)
            .collect())
    }

    async fn list_author_events(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;
        Ok(self
            .author_event_repository
            .find_by_author(&user_id, &author_id)
            .await?
            .into_iter()
            .map(AuthorEventDto::from)
            .collect())
    }

    async fn list_book_events_by_event_set_ids(
        &self,
        user_id: &str,
        event_set_ids: &[String],
    ) -> Result<Vec<BookEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let event_set_ids = event_set_ids
            .iter()
            .map(|id| EventSetId::try_from(id.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                UseCaseError::from(crate::domain::error::DomainError::Validation(error))
            })?;
        Ok(self
            .book_event_repository
            .find_by_event_set_ids(&user_id, &event_set_ids)
            .await?
            .into_iter()
            .map(BookEventDto::from)
            .collect())
    }

    async fn list_author_events_by_event_set_ids(
        &self,
        user_id: &str,
        event_set_ids: &[String],
    ) -> Result<Vec<AuthorEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let event_set_ids = event_set_ids
            .iter()
            .map(|id| EventSetId::try_from(id.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                UseCaseError::from(crate::domain::error::DomainError::Validation(error))
            })?;
        Ok(self
            .author_event_repository
            .find_by_event_set_ids(&user_id, &event_set_ids)
            .await?
            .into_iter()
            .map(AuthorEventDto::from)
            .collect())
    }

    async fn list_event_sets(&self, user_id: &str) -> Result<Vec<EventSetDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        Ok(self
            .event_set_repository
            .find_all(&user_id)
            .await?
            .into_iter()
            .map(EventSetDto::from)
            .collect())
    }

    async fn find_event_set(
        &self,
        user_id: &str,
        event_set_id: &str,
    ) -> Result<Option<EventSetDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let event_set_id = EventSetId::try_from(event_set_id).map_err(|error| {
            UseCaseError::from(crate::domain::error::DomainError::Validation(error))
        })?;
        let event_set = self
            .event_set_repository
            .find_by_id(&user_id, &event_set_id)
            .await?;
        Ok(event_set.map(EventSetDto::from))
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            entity::{
                author::AuthorId,
                book::{BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                event::{AuthorEvent, BookEvent, EventOperation, EventSetOperation},
                event_set::{EventSet, EventSetId},
                user::UserId,
            },
            repository::{
                author_event_repository::MockAuthorEventRepository,
                book_event_repository::MockBookEventRepository,
                event_set_repository::MockEventSetRepository,
            },
        },
        use_case::{
            error::UseCaseError, interactor::event::EventQueryInteractor,
            traits::event::EventQueryUseCase,
        },
    };

    fn interactor(
        book_events: MockBookEventRepository,
        author_events: MockAuthorEventRepository,
        event_sets: MockEventSetRepository,
    ) -> EventQueryInteractor<
        MockBookEventRepository,
        MockAuthorEventRepository,
        MockEventSetRepository,
    > {
        EventQueryInteractor::new(book_events, author_events, event_sets)
    }

    fn book_event(event_set_id: EventSetId, book_id: Uuid) -> BookEvent {
        BookEvent {
            event_id: 1,
            event_set_id,
            operation: EventOperation::Update,
            book_id: BookId::new(book_id).unwrap(),
            title: Some(BookTitle::new("Old Title".to_string()).unwrap()),
            author_ids: Vec::new(),
            isbn: Some(Isbn::new(String::new()).unwrap()),
            read: Some(ReadFlag::new(false)),
            owned: Some(OwnedFlag::new(false)),
            priority: Some(Priority::new(50).unwrap()),
            format: Some(BookFormat::Unknown),
            store: Some(BookStore::Unknown),
            book_created_at: Some(OffsetDateTime::UNIX_EPOCH),
            book_updated_at: Some(OffsetDateTime::UNIX_EPOCH),
            changed_at: OffsetDateTime::UNIX_EPOCH,
            extra: None,
        }
    }

    fn author_event(event_set_id: EventSetId, author_id: Uuid) -> AuthorEvent {
        AuthorEvent {
            event_id: 2,
            event_set_id,
            operation: EventOperation::Update,
            author_id: AuthorId::new(author_id),
            name: Some("Old Name".to_string()),
            yomi: Some(String::new()),
            author_created_at: Some(OffsetDateTime::UNIX_EPOCH),
            author_updated_at: Some(OffsetDateTime::UNIX_EPOCH),
            changed_at: OffsetDateTime::UNIX_EPOCH,
            extra: None,
        }
    }

    #[tokio::test]
    async fn list_event_sets_returns_empty_list() {
        let mut event_sets = MockEventSetRepository::new();
        event_sets.expect_find_all().returning(|_| Ok(Vec::new()));
        let interactor = interactor(
            MockBookEventRepository::new(),
            MockAuthorEventRepository::new(),
            event_sets,
        );

        let result = interactor.list_event_sets("user1").await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_book_events_converts_repository_result_to_dto() {
        let book_id = Uuid::new_v4();
        let mut book_events = MockBookEventRepository::new();
        book_events
            .expect_find_by_book()
            .return_once(move |_, _| Ok(vec![book_event(EventSetId::new(), book_id)]));
        let interactor = interactor(
            book_events,
            MockAuthorEventRepository::new(),
            MockEventSetRepository::new(),
        );

        let result = interactor
            .list_book_events("user1", &book_id.to_string())
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].book_id, book_id.to_string());
        assert_eq!(result[0].title.as_deref(), Some("Old Title"));
        assert_eq!(result[0].operation, "update");
    }

    #[tokio::test]
    async fn list_author_events_converts_repository_result_to_dto() {
        let author_id = Uuid::new_v4();
        let mut author_events = MockAuthorEventRepository::new();
        author_events
            .expect_find_by_author()
            .return_once(move |_, _| Ok(vec![author_event(EventSetId::new(), author_id)]));
        let interactor = interactor(
            MockBookEventRepository::new(),
            author_events,
            MockEventSetRepository::new(),
        );

        let result = interactor
            .list_author_events("user1", &author_id.to_string())
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author_id, author_id.to_string());
        assert_eq!(result[0].name.as_deref(), Some("Old Name"));
        assert_eq!(result[0].operation, "update");
    }

    #[tokio::test]
    async fn find_event_set_returns_scalar_dto_without_loading_events() {
        let event_set_id = EventSetId::new();
        let event_set_id_string = event_set_id.to_string();
        let mut event_sets = MockEventSetRepository::new();
        event_sets.expect_find_by_id().return_once(move |_, _| {
            Ok(Some(EventSet {
                id: event_set_id,
                user_id: UserId::new("user1".to_string()).unwrap(),
                operation: EventSetOperation::CreateBook,
                created_at: OffsetDateTime::UNIX_EPOCH,
            }))
        });
        let interactor = interactor(
            MockBookEventRepository::new(),
            MockAuthorEventRepository::new(),
            event_sets,
        );

        let result = interactor
            .find_event_set("user1", &event_set_id_string)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result.id, event_set_id_string);
        assert_eq!(result.operation, "create_book");
        assert_eq!(result.created_at, OffsetDateTime::UNIX_EPOCH);
    }

    #[tokio::test]
    async fn find_event_set_returns_none_when_not_found() {
        let event_set_id = EventSetId::new().to_string();
        let mut event_sets = MockEventSetRepository::new();
        event_sets.expect_find_by_id().return_once(|_, _| Ok(None));
        let interactor = interactor(
            MockBookEventRepository::new(),
            MockAuthorEventRepository::new(),
            event_sets,
        );

        let result = interactor
            .find_event_set("user1", &event_set_id)
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_book_events_by_event_set_ids_converts_batch() {
        let event_set_id = EventSetId::new();
        let expected_id = event_set_id.to_string();
        let book_id = Uuid::new_v4();
        let mut book_events = MockBookEventRepository::new();
        book_events
            .expect_find_by_event_set_ids()
            .times(1)
            .return_once(move |_, _| Ok(vec![book_event(event_set_id, book_id)]));
        let interactor = interactor(
            book_events,
            MockAuthorEventRepository::new(),
            MockEventSetRepository::new(),
        );

        let result = interactor
            .list_book_events_by_event_set_ids("user1", std::slice::from_ref(&expected_id))
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].event_set_id, expected_id);
        assert_eq!(result[0].book_id, book_id.to_string());
    }

    #[tokio::test]
    async fn list_author_events_by_event_set_ids_handles_empty_result() {
        let event_set_id = EventSetId::new().to_string();
        let mut author_events = MockAuthorEventRepository::new();
        author_events
            .expect_find_by_event_set_ids()
            .times(1)
            .return_once(|_, _| Ok(Vec::new()));
        let interactor = interactor(
            MockBookEventRepository::new(),
            author_events,
            MockEventSetRepository::new(),
        );

        let result = interactor
            .list_author_events_by_event_set_ids("user1", &[event_set_id])
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_author_events_by_event_set_ids_converts_batch() {
        let event_set_id = EventSetId::new();
        let expected_id = event_set_id.to_string();
        let author_id = Uuid::new_v4();
        let mut author_events = MockAuthorEventRepository::new();
        author_events
            .expect_find_by_event_set_ids()
            .times(1)
            .return_once(move |_, _| Ok(vec![author_event(event_set_id, author_id)]));
        let interactor = interactor(
            MockBookEventRepository::new(),
            author_events,
            MockEventSetRepository::new(),
        );

        let result = interactor
            .list_author_events_by_event_set_ids("user1", std::slice::from_ref(&expected_id))
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].event_set_id, expected_id);
        assert_eq!(result[0].author_id, author_id.to_string());
    }

    #[tokio::test]
    async fn list_book_events_by_event_set_ids_handles_empty_ids() {
        let mut book_events = MockBookEventRepository::new();
        book_events
            .expect_find_by_event_set_ids()
            .withf(|_, ids| ids.is_empty())
            .times(1)
            .return_once(|_, _| Ok(Vec::new()));
        let interactor = interactor(
            book_events,
            MockAuthorEventRepository::new(),
            MockEventSetRepository::new(),
        );

        let result = interactor
            .list_book_events_by_event_set_ids("user1", &[])
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn list_book_events_rejects_invalid_book_id() {
        let interactor = interactor(
            MockBookEventRepository::new(),
            MockAuthorEventRepository::new(),
            MockEventSetRepository::new(),
        );

        let result = interactor.list_book_events("user1", "invalid").await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn list_author_events_rejects_invalid_author_id() {
        let interactor = interactor(
            MockBookEventRepository::new(),
            MockAuthorEventRepository::new(),
            MockEventSetRepository::new(),
        );

        let result = interactor.list_author_events("user1", "invalid").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn find_event_set_rejects_invalid_id_without_repository_calls() {
        let interactor = interactor(
            MockBookEventRepository::new(),
            MockAuthorEventRepository::new(),
            MockEventSetRepository::new(),
        );

        let result = interactor.find_event_set("user1", "invalid").await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }
}
