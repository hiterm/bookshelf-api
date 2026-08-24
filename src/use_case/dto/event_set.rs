use time::OffsetDateTime;

use crate::domain::entity::event_set::EventSet;

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
