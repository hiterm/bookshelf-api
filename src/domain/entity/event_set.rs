use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entity::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSetId(Uuid);

impl EventSetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EventSetId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

impl TryFrom<&str> for EventSetId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value)
            .map(EventSetId)
            .map_err(|e| e.to_string())
    }
}

impl From<Uuid> for EventSetId {
    fn from(uuid: Uuid) -> Self {
        EventSetId(uuid)
    }
}

pub struct EventSet {
    pub id: EventSetId,
    pub user_id: UserId,
    pub operation: String,
    pub created_at: OffsetDateTime,
}
