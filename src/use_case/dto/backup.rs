use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    common::types::{BookFormat, BookStore},
    domain::entity::{
        author::{AuthorName, validate_author_yomi},
        book::{BookTitle, Isbn, Priority},
        event::{EventOperation, EventSetOperation},
    },
};

pub const CURRENT_BACKUP_FORMAT: &str = "bookshelf-current-backup";
pub const FULL_BACKUP_FORMAT: &str = "bookshelf-full-backup";
pub const BACKUP_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupAuthorV1 {
    pub id: String,
    pub name: String,
    pub yomi: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupBookV1 {
    pub id: String,
    pub title: String,
    pub isbn: String,
    pub read: bool,
    pub owned: bool,
    pub priority: i32,
    pub format: String,
    pub store: String,
    pub created_at: String,
    pub updated_at: String,
    pub author_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurrentBackupDataV1 {
    pub authors: Vec<BackupAuthorV1>,
    pub books: Vec<BackupBookV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurrentBackupV1 {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub data: CurrentBackupDataV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupEventSetV1 {
    pub id: String,
    pub operation: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupBookEventV1 {
    pub event_id: i64,
    pub event_set_id: String,
    pub operation: String,
    pub book_id: String,
    pub title: Option<String>,
    pub isbn: Option<String>,
    pub read: Option<bool>,
    pub owned: Option<bool>,
    pub priority: Option<i32>,
    pub format: Option<String>,
    pub store: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub author_ids: Vec<String>,
    pub changed_at: String,
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupAuthorEventV1 {
    pub event_id: i64,
    pub event_set_id: String,
    pub operation: String,
    pub author_id: String,
    pub name: Option<String>,
    pub yomi: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub changed_at: String,
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupHistoryV1 {
    pub event_sets: Vec<BackupEventSetV1>,
    pub book_events: Vec<BackupBookEventV1>,
    pub author_events: Vec<BackupAuthorEventV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullBackupV1 {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub data: CurrentBackupDataV1,
    pub history: BackupHistoryV1,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackupValidationError {
    #[error("unsupported backup format: {0}")]
    UnsupportedFormat(String),
    #[error("unsupported backup version: {0}")]
    UnsupportedVersion(u64),
    #[error("malformed backup: {0}")]
    Malformed(String),
    #[error("duplicate {kind} ID: {id}")]
    DuplicateId { kind: &'static str, id: String },
    #[error("invalid reference: {0}")]
    InvalidReference(String),
    #[error("unsupported value: {0}")]
    UnsupportedValue(String),
}

#[derive(Deserialize)]
struct BackupHeader {
    format: String,
    version: u64,
}

impl CurrentBackupV1 {
    pub fn parse(value: Value) -> Result<Self, BackupValidationError> {
        dispatch(&value, CURRENT_BACKUP_FORMAT)?;
        let backup: Self = serde_json::from_value(value)
            .map_err(|error| BackupValidationError::Malformed(error.to_string()))?;
        backup.validate()?;
        Ok(backup)
    }

    pub fn validate(&self) -> Result<(), BackupValidationError> {
        validate_header(&self.format, self.version, CURRENT_BACKUP_FORMAT)?;
        validate_timestamp("exportedAt", &self.exported_at)?;
        validate_current_data(&self.data)
    }
}

impl FullBackupV1 {
    pub fn parse(value: Value) -> Result<Self, BackupValidationError> {
        dispatch(&value, FULL_BACKUP_FORMAT)?;
        let backup: Self = serde_json::from_value(value)
            .map_err(|error| BackupValidationError::Malformed(error.to_string()))?;
        backup.validate()?;
        Ok(backup)
    }

    pub fn validate(&self) -> Result<(), BackupValidationError> {
        validate_header(&self.format, self.version, FULL_BACKUP_FORMAT)?;
        validate_timestamp("exportedAt", &self.exported_at)?;
        validate_current_data(&self.data)?;
        validate_history(&self.history)
    }
}

fn dispatch(value: &Value, expected_format: &str) -> Result<(), BackupValidationError> {
    let header: BackupHeader = serde_json::from_value(value.clone())
        .map_err(|error| BackupValidationError::Malformed(error.to_string()))?;
    validate_header(&header.format, header.version, expected_format)
}

fn validate_header(
    format: &str,
    version: impl Into<u64>,
    expected_format: &str,
) -> Result<(), BackupValidationError> {
    let version = version.into();
    if format != expected_format {
        return Err(BackupValidationError::UnsupportedFormat(format.to_owned()));
    }
    if version != u64::from(BACKUP_VERSION_V1) {
        return Err(BackupValidationError::UnsupportedVersion(version));
    }
    Ok(())
}

fn validate_current_data(data: &CurrentBackupDataV1) -> Result<(), BackupValidationError> {
    let mut author_ids = HashSet::new();
    for author in &data.authors {
        validate_uuid("author", &author.id)?;
        if !author_ids.insert(author.id.as_str()) {
            return Err(BackupValidationError::DuplicateId {
                kind: "author",
                id: author.id.clone(),
            });
        }
        AuthorName::new(author.name.clone())
            .map_err(|error| BackupValidationError::UnsupportedValue(error.to_string()))?;
        validate_author_yomi(author.yomi.clone())
            .map_err(|error| BackupValidationError::UnsupportedValue(error.to_string()))?;
        validate_timestamp("author.createdAt", &author.created_at)?;
        validate_timestamp("author.updatedAt", &author.updated_at)?;
    }

    let mut book_ids = HashSet::new();
    for book in &data.books {
        validate_uuid("book", &book.id)?;
        if !book_ids.insert(book.id.as_str()) {
            return Err(BackupValidationError::DuplicateId {
                kind: "book",
                id: book.id.clone(),
            });
        }
        validate_book_values(
            &book.title,
            &book.isbn,
            book.priority,
            &book.format,
            &book.store,
        )?;
        validate_timestamp("book.createdAt", &book.created_at)?;
        validate_timestamp("book.updatedAt", &book.updated_at)?;
        for author_id in &book.author_ids {
            if !author_ids.contains(author_id.as_str()) {
                return Err(BackupValidationError::InvalidReference(format!(
                    "book {} refers to unknown author {}",
                    book.id, author_id
                )));
            }
        }
    }
    Ok(())
}

fn validate_history(history: &BackupHistoryV1) -> Result<(), BackupValidationError> {
    let mut event_set_ids = HashSet::new();
    for event_set in &history.event_sets {
        validate_uuid("event set", &event_set.id)?;
        if !event_set_ids.insert(event_set.id.as_str()) {
            return Err(BackupValidationError::DuplicateId {
                kind: "event set",
                id: event_set.id.clone(),
            });
        }
        EventSetOperation::try_from(event_set.operation.as_str()).map_err(|_| {
            BackupValidationError::UnsupportedValue(format!(
                "event set operation {}",
                event_set.operation
            ))
        })?;
        validate_timestamp("eventSet.createdAt", &event_set.created_at)?;
    }

    validate_events(&history.book_events, &history.author_events, &event_set_ids)
}

fn validate_events(
    book_events: &[BackupBookEventV1],
    author_events: &[BackupAuthorEventV1],
    event_set_ids: &HashSet<&str>,
) -> Result<(), BackupValidationError> {
    let mut book_event_ids = HashSet::new();
    let mut author_event_ids = HashSet::new();
    for event in book_events {
        validate_event_common(
            "book event",
            event.event_id,
            &event.event_set_id,
            &event.operation,
            &event.changed_at,
            event_set_ids,
            &mut book_event_ids,
        )?;
        validate_uuid("book event book", &event.book_id)?;
        validate_snapshot_shape(
            &event.operation,
            event.title.is_some(),
            event.created_at.as_deref(),
            event.updated_at.as_deref(),
        )?;
        if let (Some(title), Some(isbn), Some(priority), Some(format), Some(store)) = (
            event.title.as_deref(),
            event.isbn.as_deref(),
            event.priority,
            event.format.as_deref(),
            event.store.as_deref(),
        ) {
            validate_book_values(title, isbn, priority, format, store)?;
        } else if matches!(
            event.operation.as_str(),
            "create" | "update" | "restore" | "snapshot"
        ) {
            return Err(BackupValidationError::Malformed(format!(
                "book event {} is missing snapshot fields",
                event.event_id
            )));
        }
        let mut author_ids = HashSet::new();
        for author_id in &event.author_ids {
            validate_uuid("book event author", author_id)?;
            if !author_ids.insert(author_id) {
                return Err(BackupValidationError::DuplicateId {
                    kind: "book event author",
                    id: author_id.clone(),
                });
            }
        }
        validate_extra(&event.operation, event.extra.as_ref())?;
    }
    for event in author_events {
        validate_event_common(
            "author event",
            event.event_id,
            &event.event_set_id,
            &event.operation,
            &event.changed_at,
            event_set_ids,
            &mut author_event_ids,
        )?;
        validate_uuid("author event author", &event.author_id)?;
        validate_snapshot_shape(
            &event.operation,
            event.name.is_some(),
            event.created_at.as_deref(),
            event.updated_at.as_deref(),
        )?;
        if let Some(name) = &event.name {
            AuthorName::new(name.clone())
                .map_err(|error| BackupValidationError::UnsupportedValue(error.to_string()))?;
        }
        if let Some(yomi) = &event.yomi {
            validate_author_yomi(yomi.clone())
                .map_err(|error| BackupValidationError::UnsupportedValue(error.to_string()))?;
        }
        validate_extra(&event.operation, event.extra.as_ref())?;
    }

    for event in book_events {
        validate_restore_reference(
            event.event_id,
            &event.operation,
            event.extra.as_ref(),
            &book_event_ids,
        )?;
    }
    for event in author_events {
        validate_restore_reference(
            event.event_id,
            &event.operation,
            event.extra.as_ref(),
            &author_event_ids,
        )?;
    }
    Ok(())
}

fn validate_event_common<'a>(
    kind: &'static str,
    event_id: i64,
    event_set_id: &'a str,
    operation: &str,
    changed_at: &str,
    event_set_ids: &HashSet<&'a str>,
    event_ids: &mut HashSet<i64>,
) -> Result<(), BackupValidationError> {
    if !event_ids.insert(event_id) {
        return Err(BackupValidationError::DuplicateId {
            kind,
            id: event_id.to_string(),
        });
    }
    if !event_set_ids.contains(event_set_id) {
        return Err(BackupValidationError::InvalidReference(format!(
            "event {} refers to unknown event set {}",
            event_id, event_set_id
        )));
    }
    EventOperation::try_from(operation).map_err(|_| {
        BackupValidationError::UnsupportedValue(format!("event operation {operation}"))
    })?;
    validate_timestamp("event.changedAt", changed_at)
}

fn validate_snapshot_shape(
    operation: &str,
    has_primary_field: bool,
    created_at: Option<&str>,
    updated_at: Option<&str>,
) -> Result<(), BackupValidationError> {
    let requires_snapshot = matches!(operation, "create" | "update" | "restore" | "snapshot");
    if requires_snapshot != has_primary_field
        || requires_snapshot != created_at.is_some()
        || requires_snapshot != updated_at.is_some()
    {
        return Err(BackupValidationError::Malformed(format!(
            "event snapshot fields do not match operation {operation}"
        )));
    }
    if let Some(value) = created_at {
        validate_timestamp("event.createdAt", value)?;
    }
    if let Some(value) = updated_at {
        validate_timestamp("event.updatedAt", value)?;
    }
    Ok(())
}

fn validate_extra(operation: &str, extra: Option<&Value>) -> Result<(), BackupValidationError> {
    match (operation, extra) {
        ("restore", Some(Value::Object(object)))
            if object.get("version").and_then(Value::as_u64) == Some(1)
                && object
                    .get("source_event_id")
                    .and_then(Value::as_i64)
                    .is_some()
                && object.len() == 2 =>
        {
            Ok(())
        }
        ("restore", _) => Err(BackupValidationError::Malformed(
            "restore extra must have version 1 and source_event_id".to_string(),
        )),
        ("snapshot", Some(Value::Object(object)))
            if object.get("version").and_then(Value::as_u64) == Some(1)
                && object.get("reason").and_then(Value::as_str)
                    == Some("current_backup_restore")
                && matches!(
                    object.get("phase").and_then(Value::as_str),
                    Some("before" | "after")
                )
                && object.len() == 3 =>
        {
            Ok(())
        }
        ("delete", Some(Value::Object(object)))
            if object.get("type").and_then(Value::as_str) == Some("merge")
                && object.get("version").and_then(Value::as_u64) == Some(1)
                && object
                    .get("destination_author_id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| Uuid::parse_str(id).is_ok())
                && object.len() == 3 =>
        {
            Ok(())
        }
        ("merge_as_destination", Some(Value::Object(object)))
            if object.get("version").and_then(Value::as_u64) == Some(1)
                && object
                    .get("source_author_id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| Uuid::parse_str(id).is_ok())
                && object.len() == 2 =>
        {
            Ok(())
        }
        (_, None) => Ok(()),
        _ => Err(BackupValidationError::UnsupportedValue(
            "unsupported event extra schema".to_string(),
        )),
    }
}

fn validate_restore_reference(
    event_id: i64,
    operation: &str,
    extra: Option<&Value>,
    event_ids: &HashSet<i64>,
) -> Result<(), BackupValidationError> {
    if operation == "restore" {
        let source = extra
            .and_then(|value| value.get("source_event_id"))
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                BackupValidationError::Malformed("missing restore source".to_string())
            })?;
        if !event_ids.contains(&source) {
            return Err(BackupValidationError::InvalidReference(format!(
                "event {event_id} refers to unknown source event {source}"
            )));
        }
    }
    Ok(())
}

fn validate_uuid(kind: &str, value: &str) -> Result<(), BackupValidationError> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| BackupValidationError::Malformed(format!("invalid {kind} UUID: {value}")))
}

fn validate_book_values(
    title: &str,
    isbn: &str,
    priority: i32,
    format: &str,
    store: &str,
) -> Result<(), BackupValidationError> {
    BookTitle::new(title.to_string())
        .map_err(|error| BackupValidationError::UnsupportedValue(error.to_string()))?;
    Isbn::new(isbn.to_string())
        .map_err(|error| BackupValidationError::UnsupportedValue(error.to_string()))?;
    Priority::new(priority)
        .map_err(|error| BackupValidationError::UnsupportedValue(error.to_string()))?;
    BookFormat::try_from(format)
        .map_err(|_| BackupValidationError::UnsupportedValue(format!("book format {format}")))?;
    BookStore::try_from(store)
        .map_err(|_| BackupValidationError::UnsupportedValue(format!("book store {store}")))?;
    Ok(())
}

fn validate_timestamp(kind: &str, value: &str) -> Result<(), BackupValidationError> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        .map(|_| ())
        .map_err(|_| BackupValidationError::Malformed(format!("invalid {kind}: {value}")))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn current() -> Value {
        json!({
            "format": CURRENT_BACKUP_FORMAT,
            "version": 1,
            "exportedAt": "2026-08-26T00:00:00Z",
            "data": {
                "authors": [{
                    "id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
                    "name": "Author", "yomi": "", "createdAt": "2026-08-25T00:00:00Z",
                    "updatedAt": "2026-08-25T00:00:00Z"
                }],
                "books": [{
                    "id": "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "title": "Book", "isbn": "",
                    "read": false, "owned": true, "priority": 1, "format": "Printed",
                    "store": "Unknown", "createdAt": "2026-08-25T00:00:00Z",
                    "updatedAt": "2026-08-25T00:00:00Z",
                    "authorIds": ["aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"]
                }]
            }
        })
    }

    #[test]
    fn current_v1_round_trips_camel_case_contract() {
        let backup = CurrentBackupV1::parse(current()).unwrap();
        let value = serde_json::to_value(backup).unwrap();
        assert!(value.get("exportedAt").is_some());
        assert!(value["data"]["books"][0].get("authorIds").is_some());
        assert!(value["data"]["books"][0].get("userId").is_none());
    }

    #[test]
    fn rejects_unknown_version_before_full_deserialization() {
        let mut value = current();
        value["version"] = json!(2);
        assert_eq!(
            CurrentBackupV1::parse(value).unwrap_err(),
            BackupValidationError::UnsupportedVersion(2)
        );
    }

    #[test]
    fn rejects_duplicate_author_id() {
        let mut value = current();
        let author = value["data"]["authors"][0].clone();
        value["data"]["authors"]
            .as_array_mut()
            .unwrap()
            .push(author);
        assert!(matches!(
            CurrentBackupV1::parse(value),
            Err(BackupValidationError::DuplicateId { kind: "author", .. })
        ));
    }

    #[test]
    fn rejects_unknown_author_reference() {
        let mut value = current();
        value["data"]["books"][0]["authorIds"] = json!(["cccccccc-cccc-4ccc-8ccc-cccccccccccc"]);
        assert!(matches!(
            CurrentBackupV1::parse(value),
            Err(BackupValidationError::InvalidReference(_))
        ));
    }

    #[test]
    fn nullable_delete_event_snapshot_is_valid() {
        let backup = FullBackupV1 {
            format: FULL_BACKUP_FORMAT.to_string(),
            version: 1,
            exported_at: "2026-08-26T00:00:00Z".to_string(),
            data: CurrentBackupDataV1 {
                authors: vec![],
                books: vec![],
            },
            history: BackupHistoryV1 {
                event_sets: vec![BackupEventSetV1 {
                    id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd".to_string(),
                    operation: "delete_book".to_string(),
                    created_at: "2026-08-26T00:00:00Z".to_string(),
                }],
                book_events: vec![BackupBookEventV1 {
                    event_id: 1,
                    event_set_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd".to_string(),
                    operation: "delete".to_string(),
                    book_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb".to_string(),
                    title: None,
                    isbn: None,
                    read: None,
                    owned: None,
                    priority: None,
                    format: None,
                    store: None,
                    created_at: None,
                    updated_at: None,
                    author_ids: vec![],
                    changed_at: "2026-08-26T00:00:00Z".to_string(),
                    extra: None,
                }],
                author_events: vec![],
            },
        };
        backup.validate().unwrap();
    }

    #[test]
    fn unknown_version_one_extra_is_rejected() {
        let mut backup = FullBackupV1 {
            format: FULL_BACKUP_FORMAT.to_string(),
            version: 1,
            exported_at: "2026-08-26T00:00:00Z".to_string(),
            data: CurrentBackupDataV1 {
                authors: vec![],
                books: vec![],
            },
            history: BackupHistoryV1 {
                event_sets: vec![BackupEventSetV1 {
                    id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd".to_string(),
                    operation: "delete_book".to_string(),
                    created_at: "2026-08-26T00:00:00Z".to_string(),
                }],
                book_events: vec![BackupBookEventV1 {
                    event_id: 1,
                    event_set_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd".to_string(),
                    operation: "delete".to_string(),
                    book_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb".to_string(),
                    title: None,
                    isbn: None,
                    read: None,
                    owned: None,
                    priority: None,
                    format: None,
                    store: None,
                    created_at: None,
                    updated_at: None,
                    author_ids: vec![],
                    changed_at: "2026-08-26T00:00:00Z".to_string(),
                    extra: None,
                }],
                author_events: vec![],
            },
        };
        backup.history.book_events[0].extra = Some(json!({"version": 1, "unknown": true}));

        assert!(matches!(
            backup.validate(),
            Err(BackupValidationError::UnsupportedValue(_))
        ));
    }
}
