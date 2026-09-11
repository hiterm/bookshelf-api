use serde::Serialize;

use crate::use_case::port::backup::{
    BackupAuthorProjection, BackupAuthorRevisionProjection, BackupAuthorSnapshotProjection,
    BackupBookProjection, BackupBookRevisionProjection, BackupBookSnapshotProjection,
    BackupChangesProjection, BackupEntityChangeProjection, BackupHistoryProjection,
    BackupOperationProjection, BackupProjection,
};

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

impl From<BackupEntityChangeProjection> for BackupEntityChange {
    fn from(value: BackupEntityChangeProjection) -> Self {
        Self {
            book_id: value.book_id,
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

impl From<BackupProjection> for BackupData {
    fn from(value: BackupProjection) -> Self {
        Self {
            authors: value.authors.into_iter().map(Into::into).collect(),
            books: value.books.into_iter().map(Into::into).collect(),
            history: value.history.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        BackupAuthor, BackupAuthorRevision, BackupAuthorSnapshot, BackupBook, BackupBookRevision,
        BackupBookSnapshot, BackupChanges, BackupData, BackupEntityChange, BackupEnvelope,
        BackupHistory, BackupOperation, BackupScope,
    };

    fn book_snapshot(title: &str) -> BackupBookSnapshot {
        BackupBookSnapshot {
            title: title.to_owned(),
            author_ids: vec!["author-1".to_owned()],
            isbn: "9780000000000".to_owned(),
            read: true,
            owned: false,
            priority: 42,
            format: "E_BOOK".to_owned(),
            store: "KINDLE".to_owned(),
            purchase_date: Some("2026-09-10".to_owned()),
            created_at: "2026-09-10T01:00:00Z".to_owned(),
            updated_at: "2026-09-10T02:00:00Z".to_owned(),
        }
    }

    #[test]
    fn full_envelope_matches_the_complete_v1_json_contract() {
        let actual = serde_json::to_value(BackupEnvelope::new(
            BackupScope::Full,
            "2026-09-11T03:00:00Z".to_owned(),
            BackupData {
                authors: vec![BackupAuthor {
                    id: "author-1".to_owned(),
                    name: "Author".to_owned(),
                    yomi: "オーサー".to_owned(),
                    created_at: "2026-09-10T00:00:00Z".to_owned(),
                    updated_at: "2026-09-10T00:30:00Z".to_owned(),
                }],
                books: vec![BackupBook {
                    id: "book-1".to_owned(),
                    snapshot: book_snapshot("Current Book"),
                }],
                history: Some(BackupHistory {
                    operations: vec![BackupOperation {
                        id: "operation-2".to_owned(),
                        operation_type: "undo".to_owned(),
                        detail: Some(json!({"reason": "fixture"})),
                        undo_of_operation_id: Some("operation-1".to_owned()),
                        created_at: "2026-09-10T02:00:00Z".to_owned(),
                        changes: BackupChanges {
                            books: vec![BackupEntityChange {
                                book_id: Some("book-1".to_owned()),
                                author_id: None,
                                before_revision_number: None,
                                after_revision_number: Some(2),
                            }],
                            authors: vec![BackupEntityChange {
                                book_id: None,
                                author_id: Some("author-1".to_owned()),
                                before_revision_number: Some(1),
                                after_revision_number: None,
                            }],
                        },
                    }],
                    book_revisions: vec![BackupBookRevision {
                        book_id: "book-1".to_owned(),
                        revision_number: 2,
                        snapshot: book_snapshot("Revision Book"),
                        recorded_at: "2026-09-10T02:00:01Z".to_owned(),
                    }],
                    author_revisions: vec![BackupAuthorRevision {
                        author_id: "author-1".to_owned(),
                        revision_number: 1,
                        snapshot: BackupAuthorSnapshot {
                            name: "Author".to_owned(),
                            yomi: "オーサー".to_owned(),
                            created_at: "2026-09-10T00:00:00Z".to_owned(),
                            updated_at: "2026-09-10T00:30:00Z".to_owned(),
                        },
                        recorded_at: "2026-09-10T00:30:01Z".to_owned(),
                    }],
                }),
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
        let actual = serde_json::to_value(BackupEnvelope::new(
            BackupScope::Snapshot,
            "2026-09-11T03:00:00Z".to_owned(),
            BackupData {
                authors: vec![],
                books: vec![],
                history: None,
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
