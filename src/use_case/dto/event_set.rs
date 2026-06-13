use time::OffsetDateTime;

use crate::{
    domain::entity::event_set::EventSet,
    use_case::dto::event::{AuthorEventDto, BookEventDto},
};

#[derive(Debug, Clone)]
pub struct EventSetDto {
    pub id: String,
    pub operation: String,
    pub created_at: OffsetDateTime,
}

impl From<EventSet> for EventSetDto {
    fn from(e: EventSet) -> Self {
        Self {
            id: e.id.to_string(),
            operation: e.operation.as_str().to_string(),
            created_at: e.created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventSetDetailDto {
    pub id: String,
    pub operation: String,
    pub created_at: OffsetDateTime,
    pub book_events: Vec<BookEventDto>,
    pub author_events: Vec<AuthorEventDto>,
}

impl EventSetDetailDto {
    pub fn new(
        event_set: EventSet,
        book_events: Vec<BookEventDto>,
        author_events: Vec<AuthorEventDto>,
    ) -> Self {
        Self {
            id: event_set.id.to_string(),
            operation: event_set.operation.as_str().to_string(),
            created_at: event_set.created_at,
            book_events,
            author_events,
        }
    }
}
