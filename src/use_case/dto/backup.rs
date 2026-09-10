use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAuthor {
    pub id: String,
    pub name: String,
    pub yomi: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupBookSnapshot {
    pub title: String,
    pub author_ids: Vec<String>,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: String,
    pub store: String,
    pub purchase_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupBook {
    pub id: String,
    #[serde(flatten)]
    pub snapshot: BackupBookSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupBookRevision {
    pub book_id: String,
    pub revision_number: i32,
    pub snapshot: BackupBookSnapshot,
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAuthorSnapshot {
    pub name: String,
    pub yomi: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAuthorRevision {
    pub author_id: String,
    pub revision_number: i32,
    pub snapshot: BackupAuthorSnapshot,
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntityChange {
    #[serde(rename = "bookId", skip_serializing_if = "Option::is_none")]
    pub book_id: Option<String>,
    #[serde(rename = "authorId", skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct BackupChanges {
    pub books: Vec<BackupEntityChange>,
    pub authors: Vec<BackupEntityChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupOperation {
    pub id: String,
    #[serde(rename = "type")]
    pub operation_type: String,
    pub detail: Option<serde_json::Value>,
    pub undo_of_operation_id: Option<String>,
    pub created_at: String,
    pub changes: BackupChanges,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupHistory {
    pub operations: Vec<BackupOperation>,
    pub book_revisions: Vec<BackupBookRevision>,
    pub author_revisions: Vec<BackupAuthorRevision>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BackupData {
    pub authors: Vec<BackupAuthor>,
    pub books: Vec<BackupBook>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<BackupHistory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BackupScope {
    Snapshot,
    Full,
}

impl BackupScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Snapshot => "snapshot",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEnvelope {
    pub format: &'static str,
    pub version: u32,
    pub scope: BackupScope,
    pub exported_at: String,
    pub data: BackupData,
}

impl BackupEnvelope {
    pub fn new(scope: BackupScope, exported_at: String, data: BackupData) -> Self {
        Self {
            format: "bookshelf-backup",
            version: 1,
            scope,
            exported_at,
            data,
        }
    }
}
