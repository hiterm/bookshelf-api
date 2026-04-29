use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entity::user::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSetId(Uuid);

impl ChangeSetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ChangeSetId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ChangeSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

impl TryFrom<&str> for ChangeSetId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value)
            .map(ChangeSetId)
            .map_err(|e| e.to_string())
    }
}

impl From<Uuid> for ChangeSetId {
    fn from(uuid: Uuid) -> Self {
        ChangeSetId(uuid)
    }
}

pub struct ChangeSet {
    pub id: ChangeSetId,
    pub user_id: UserId,
    pub operation: String,
    pub created_at: OffsetDateTime,
}
