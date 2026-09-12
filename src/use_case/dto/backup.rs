use serde::{Serialize, Serializer, ser::Error};
use time::{Date, OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    common::types::{BookFormat, BookStore},
    use_case::port::backup::{
        BackupAuthorChangeProjection, BackupAuthorProjection, BackupAuthorRevisionProjection,
        BackupAuthorSnapshotProjection, BackupBookChangeProjection, BackupBookProjection,
        BackupBookRevisionProjection, BackupBookSnapshotProjection, BackupChangesProjection,
        BackupFullProjection, BackupHistoryProjection, BackupOperationProjection,
        BackupSnapshotProjection,
    },
};

const BACKUP_FORMAT: &str = "bookshelf-backup";
const BACKUP_VERSION: u32 = 1;

fn serialize_timestamp<S>(value: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.format(&Rfc3339).map_err(S::Error::custom)?)
}

fn serialize_optional_date<S>(value: &Option<Date>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(&value.to_string()),
        None => serializer.serialize_none(),
    }
}

fn serialize_book_format<S>(value: &BookFormat, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(match value {
        BookFormat::EBook => "E_BOOK",
        BookFormat::Printed => "PRINTED",
        BookFormat::Unknown => "UNKNOWN",
    })
}

fn serialize_book_store<S>(value: &BookStore, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(match value {
        BookStore::Kindle => "KINDLE",
        BookStore::Unknown => "UNKNOWN",
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAuthor {
    pub id: String,
    pub name: String,
    pub yomi: String,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: OffsetDateTime,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: OffsetDateTime,
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
    #[serde(serialize_with = "serialize_book_format")]
    pub format: BookFormat,
    #[serde(serialize_with = "serialize_book_store")]
    pub store: BookStore,
    #[serde(serialize_with = "serialize_optional_date")]
    pub purchase_date: Option<Date>,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: OffsetDateTime,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: OffsetDateTime,
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
    #[serde(serialize_with = "serialize_timestamp")]
    pub recorded_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAuthorSnapshot {
    pub name: String,
    pub yomi: String,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: OffsetDateTime,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAuthorRevision {
    pub author_id: String,
    pub revision_number: i32,
    pub snapshot: BackupAuthorSnapshot,
    #[serde(serialize_with = "serialize_timestamp")]
    pub recorded_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupBookChange {
    pub book_id: String,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAuthorChange {
    pub author_id: String,
    pub before_revision_number: Option<i32>,
    pub after_revision_number: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct BackupChanges {
    pub books: Vec<BackupBookChange>,
    pub authors: Vec<BackupAuthorChange>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupOperation {
    pub id: String,
    #[serde(rename = "type")]
    pub operation_type: String,
    pub detail: Option<serde_json::Value>,
    pub undo_of_operation_id: Option<String>,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: OffsetDateTime,
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
pub struct BackupSnapshotData {
    pub authors: Vec<BackupAuthor>,
    pub books: Vec<BackupBook>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BackupFullData {
    pub authors: Vec<BackupAuthor>,
    pub books: Vec<BackupBook>,
    pub history: BackupHistory,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSnapshotEnvelope {
    format: &'static str,
    version: u32,
    scope: &'static str,
    #[serde(serialize_with = "serialize_timestamp")]
    exported_at: OffsetDateTime,
    data: BackupSnapshotData,
}

impl BackupSnapshotEnvelope {
    pub fn new(exported_at: OffsetDateTime, data: BackupSnapshotData) -> Self {
        Self {
            format: BACKUP_FORMAT,
            version: BACKUP_VERSION,
            scope: "snapshot",
            exported_at,
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFullEnvelope {
    format: &'static str,
    version: u32,
    scope: &'static str,
    #[serde(serialize_with = "serialize_timestamp")]
    exported_at: OffsetDateTime,
    data: BackupFullData,
}

impl BackupFullEnvelope {
    pub fn new(exported_at: OffsetDateTime, data: BackupFullData) -> Self {
        Self {
            format: BACKUP_FORMAT,
            version: BACKUP_VERSION,
            scope: "full",
            exported_at,
            data,
        }
    }
}

impl From<BackupAuthorProjection> for BackupAuthor {
    fn from(value: BackupAuthorProjection) -> Self {
        Self {
            id: value.id,
            name: value.name,
            yomi: value.yomi,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<BackupBookSnapshotProjection> for BackupBookSnapshot {
    fn from(value: BackupBookSnapshotProjection) -> Self {
        Self {
            title: value.title,
            author_ids: value.author_ids,
            isbn: value.isbn,
            read: value.read,
            owned: value.owned,
            priority: value.priority,
            format: value.format,
            store: value.store,
            purchase_date: value.purchase_date,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<BackupBookProjection> for BackupBook {
    fn from(value: BackupBookProjection) -> Self {
        Self {
            id: value.id,
            snapshot: value.snapshot.into(),
        }
    }
}

impl From<BackupBookRevisionProjection> for BackupBookRevision {
    fn from(value: BackupBookRevisionProjection) -> Self {
        Self {
            book_id: value.book_id,
            revision_number: value.revision_number,
            snapshot: value.snapshot.into(),
            recorded_at: value.recorded_at,
        }
    }
}

impl From<BackupAuthorSnapshotProjection> for BackupAuthorSnapshot {
    fn from(value: BackupAuthorSnapshotProjection) -> Self {
        Self {
            name: value.name,
            yomi: value.yomi,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<BackupAuthorRevisionProjection> for BackupAuthorRevision {
    fn from(value: BackupAuthorRevisionProjection) -> Self {
        Self {
            author_id: value.author_id,
            revision_number: value.revision_number,
            snapshot: value.snapshot.into(),
            recorded_at: value.recorded_at,
        }
    }
}

impl From<BackupBookChangeProjection> for BackupBookChange {
    fn from(value: BackupBookChangeProjection) -> Self {
        Self {
            book_id: value.book_id,
            before_revision_number: value.before_revision_number,
            after_revision_number: value.after_revision_number,
        }
    }
}

impl From<BackupAuthorChangeProjection> for BackupAuthorChange {
    fn from(value: BackupAuthorChangeProjection) -> Self {
        Self {
            author_id: value.author_id,
            before_revision_number: value.before_revision_number,
            after_revision_number: value.after_revision_number,
        }
    }
}

impl From<BackupChangesProjection> for BackupChanges {
    fn from(value: BackupChangesProjection) -> Self {
        Self {
            books: value.books.into_iter().map(Into::into).collect(),
            authors: value.authors.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BackupOperationProjection> for BackupOperation {
    fn from(value: BackupOperationProjection) -> Self {
        Self {
            id: value.id,
            operation_type: value.operation_type,
            detail: value.detail,
            undo_of_operation_id: value.undo_of_operation_id,
            created_at: value.created_at,
            changes: value.changes.into(),
        }
    }
}

impl From<BackupHistoryProjection> for BackupHistory {
    fn from(value: BackupHistoryProjection) -> Self {
        Self {
            operations: value.operations.into_iter().map(Into::into).collect(),
            book_revisions: value.book_revisions.into_iter().map(Into::into).collect(),
            author_revisions: value.author_revisions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BackupSnapshotProjection> for BackupSnapshotData {
    fn from(value: BackupSnapshotProjection) -> Self {
        Self {
            authors: value.authors.into_iter().map(Into::into).collect(),
            books: value.books.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<BackupFullProjection> for BackupFullData {
    fn from(value: BackupFullProjection) -> Self {
        Self {
            authors: value.authors.into_iter().map(Into::into).collect(),
            books: value.books.into_iter().map(Into::into).collect(),
            history: value.history.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::{Date, Month, OffsetDateTime, format_description::well_known::Rfc3339};

    use crate::common::types::{BookFormat, BookStore};

    use super::{
        BackupAuthor, BackupAuthorChange, BackupAuthorRevision, BackupAuthorSnapshot, BackupBook,
        BackupBookChange, BackupBookRevision, BackupBookSnapshot, BackupChanges, BackupFullData,
        BackupFullEnvelope, BackupHistory, BackupOperation, BackupSnapshotData,
        BackupSnapshotEnvelope,
    };

    fn timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).expect("valid timestamp")
    }

    fn book_snapshot(title: &str) -> BackupBookSnapshot {
        BackupBookSnapshot {
            title: title.to_owned(),
            author_ids: vec!["author-1".to_owned()],
            isbn: "9780000000000".to_owned(),
            read: true,
            owned: false,
            priority: 42,
            format: BookFormat::EBook,
            store: BookStore::Kindle,
            purchase_date: Some(Date::from_calendar_date(2026, Month::September, 10).unwrap()),
            created_at: timestamp("2026-09-10T01:00:00Z"),
            updated_at: timestamp("2026-09-10T02:00:00Z"),
        }
    }

    #[test]
    fn full_envelope_matches_the_complete_v1_json_contract() {
        let actual = serde_json::to_value(BackupFullEnvelope::new(
            timestamp("2026-09-11T03:00:00Z"),
            BackupFullData {
                authors: vec![BackupAuthor {
                    id: "author-1".to_owned(),
                    name: "Author".to_owned(),
                    yomi: "オーサー".to_owned(),
                    created_at: timestamp("2026-09-10T00:00:00Z"),
                    updated_at: timestamp("2026-09-10T00:30:00Z"),
                }],
                books: vec![BackupBook {
                    id: "book-1".to_owned(),
                    snapshot: book_snapshot("Current Book"),
                }],
                history: BackupHistory {
                    operations: vec![BackupOperation {
                        id: "operation-2".to_owned(),
                        operation_type: "undo".to_owned(),
                        detail: Some(json!({"reason": "fixture"})),
                        undo_of_operation_id: Some("operation-1".to_owned()),
                        created_at: timestamp("2026-09-10T02:00:00Z"),
                        changes: BackupChanges {
                            books: vec![BackupBookChange {
                                book_id: "book-1".to_owned(),
                                before_revision_number: None,
                                after_revision_number: Some(2),
                            }],
                            authors: vec![BackupAuthorChange {
                                author_id: "author-1".to_owned(),
                                before_revision_number: Some(1),
                                after_revision_number: None,
                            }],
                        },
                    }],
                    book_revisions: vec![BackupBookRevision {
                        book_id: "book-1".to_owned(),
                        revision_number: 2,
                        snapshot: book_snapshot("Revision Book"),
                        recorded_at: timestamp("2026-09-10T02:00:01Z"),
                    }],
                    author_revisions: vec![BackupAuthorRevision {
                        author_id: "author-1".to_owned(),
                        revision_number: 1,
                        snapshot: BackupAuthorSnapshot {
                            name: "Author".to_owned(),
                            yomi: "オーサー".to_owned(),
                            created_at: timestamp("2026-09-10T00:00:00Z"),
                            updated_at: timestamp("2026-09-10T00:30:00Z"),
                        },
                        recorded_at: timestamp("2026-09-10T00:30:01Z"),
                    }],
                },
            },
        ))
        .expect("backup envelope is serializable");

        let expected = json!({
            "format": "bookshelf-backup",
            "version": 1,
            "scope": "full",
            "exportedAt": "2026-09-11T03:00:00Z",
            "data": {
                "authors": [{
                    "id": "author-1",
                    "name": "Author",
                    "yomi": "オーサー",
                    "createdAt": "2026-09-10T00:00:00Z",
                    "updatedAt": "2026-09-10T00:30:00Z"
                }],
                "books": [{
                    "id": "book-1",
                    "title": "Current Book",
                    "authorIds": ["author-1"],
                    "isbn": "9780000000000",
                    "read": true,
                    "owned": false,
                    "priority": 42,
                    "format": "E_BOOK",
                    "store": "KINDLE",
                    "purchaseDate": "2026-09-10",
                    "createdAt": "2026-09-10T01:00:00Z",
                    "updatedAt": "2026-09-10T02:00:00Z"
                }],
                "history": {
                    "operations": [{
                        "id": "operation-2",
                        "type": "undo",
                        "detail": {"reason": "fixture"},
                        "undoOfOperationId": "operation-1",
                        "createdAt": "2026-09-10T02:00:00Z",
                        "changes": {
                            "books": [{
                                "bookId": "book-1",
                                "beforeRevisionNumber": null,
                                "afterRevisionNumber": 2
                            }],
                            "authors": [{
                                "authorId": "author-1",
                                "beforeRevisionNumber": 1,
                                "afterRevisionNumber": null
                            }]
                        }
                    }],
                    "bookRevisions": [{
                        "bookId": "book-1",
                        "revisionNumber": 2,
                        "snapshot": {
                            "title": "Revision Book",
                            "authorIds": ["author-1"],
                            "isbn": "9780000000000",
                            "read": true,
                            "owned": false,
                            "priority": 42,
                            "format": "E_BOOK",
                            "store": "KINDLE",
                            "purchaseDate": "2026-09-10",
                            "createdAt": "2026-09-10T01:00:00Z",
                            "updatedAt": "2026-09-10T02:00:00Z"
                        },
                        "recordedAt": "2026-09-10T02:00:01Z"
                    }],
                    "authorRevisions": [{
                        "authorId": "author-1",
                        "revisionNumber": 1,
                        "snapshot": {
                            "name": "Author",
                            "yomi": "オーサー",
                            "createdAt": "2026-09-10T00:00:00Z",
                            "updatedAt": "2026-09-10T00:30:00Z"
                        },
                        "recordedAt": "2026-09-10T00:30:01Z"
                    }]
                }
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn snapshot_omits_the_history_key() {
        let actual = serde_json::to_value(BackupSnapshotEnvelope::new(
            timestamp("2026-09-11T03:00:00Z"),
            BackupSnapshotData {
                authors: vec![],
                books: vec![],
            },
        ))
        .expect("backup envelope is serializable");

        assert_eq!(
            actual,
            json!({
                "format": "bookshelf-backup",
                "version": 1,
                "scope": "snapshot",
                "exportedAt": "2026-09-11T03:00:00Z",
                "data": {"authors": [], "books": []}
            })
        );
    }
}
