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
            event_set::{EventSetDetailDto, EventSetDto},
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
    ) -> Result<Option<EventSetDetailDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let event_set_id = EventSetId::try_from(event_set_id).map_err(|error| {
            UseCaseError::from(crate::domain::error::DomainError::Unexpected(error))
        })?;
        let Some(event_set) = self
            .event_set_repository
            .find_by_id(&user_id, &event_set_id)
            .await?
        else {
            return Ok(None);
        };
        let book_events = self
            .book_event_repository
            .find_by_event_set(&user_id, &event_set_id)
            .await?
            .into_iter()
            .map(BookEventDto::from)
            .collect();
        let author_events = self
            .author_event_repository
            .find_by_event_set(&user_id, &event_set_id)
            .await?
            .into_iter()
            .map(AuthorEventDto::from)
            .collect();
        Ok(Some(EventSetDetailDto::new(
            event_set,
            book_events,
            author_events,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::repository::{
            author_event_repository::MockAuthorEventRepository,
            book_event_repository::MockBookEventRepository,
            event_set_repository::MockEventSetRepository,
        },
        use_case::{interactor::event::EventQueryInteractor, traits::event::EventQueryUseCase},
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
    async fn list_book_events_rejects_invalid_book_id() {
        let interactor = interactor(
            MockBookEventRepository::new(),
            MockAuthorEventRepository::new(),
            MockEventSetRepository::new(),
        );

        let result = interactor.list_book_events("user1", "invalid").await;

        assert!(result.is_err());
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

        assert!(result.is_err());
    }
}
