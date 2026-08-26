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
    },
};

pub const SNAPSHOT_BACKUP_FORMAT: &str = "bookshelf-snapshot-backup";
pub const FULL_BACKUP_FORMAT: &str = "bookshelf-full-backup";
pub const BACKUP_VERSION_V1: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupValidationIssue {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupValidationSummary {
    pub books: usize,
    pub authors: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupValidationResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<BackupValidationSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BackupValidationIssue>,
}

#[derive(Debug)]
pub struct BackupValidation<T> {
    pub backup: Option<T>,
    pub response: BackupValidationResponse,
}

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
pub struct SnapshotBackupDataV1 {
    pub authors: Vec<BackupAuthorV1>,
    pub books: Vec<BackupBookV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotBackupV1 {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub data: SnapshotBackupDataV1,
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
    pub data: SnapshotBackupDataV1,
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

impl SnapshotBackupV1 {
    pub fn parse(value: Value) -> Result<Self, BackupValidationError> {
        dispatch(&value, SNAPSHOT_BACKUP_FORMAT)?;
        let backup: Self = serde_json::from_value(value)
            .map_err(|error| BackupValidationError::Malformed(error.to_string()))?;
        backup.validate()?;
        Ok(backup)
    }

    pub fn validate(&self) -> Result<(), BackupValidationError> {
        validate_header(&self.format, self.version, SNAPSHOT_BACKUP_FORMAT)?;
        validate_timestamp("exportedAt", &self.exported_at)?;
        validate_snapshot_data(&self.data)
    }
}

pub fn validate_snapshot_backup(value: Value) -> BackupValidation<SnapshotBackupV1> {
    let format = value
        .get("format")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let backup = match serde_json::from_value::<SnapshotBackupV1>(value) {
        Ok(backup) => backup,
        Err(error) => {
            return invalid_validation(
                format,
                version,
                "invalid_structure",
                "$",
                error.to_string(),
            );
        }
    };
    if let Err(error) = backup.validate() {
        let (code, path) = validation_error_location(&error);
        return invalid_validation(format, version, code, path, error.to_string());
    }
    BackupValidation {
        response: BackupValidationResponse {
            valid: true,
            format,
            version,
            summary: Some(BackupValidationSummary {
                books: backup.data.books.len(),
                authors: backup.data.authors.len(),
            }),
            errors: vec![],
        },
        backup: Some(backup),
    }
}

fn invalid_validation<T>(
    format: Option<String>,
    version: Option<u32>,
    code: &'static str,
    path: &str,
    message: String,
) -> BackupValidation<T> {
    BackupValidation {
        backup: None,
        response: BackupValidationResponse {
            valid: false,
            format,
            version,
            summary: None,
            errors: vec![BackupValidationIssue {
                code,
                path: path.to_string(),
                message,
            }],
        },
    }
}

fn validation_error_location(error: &BackupValidationError) -> (&'static str, &'static str) {
    match error {
        BackupValidationError::UnsupportedFormat(_) => ("invalid_format", "format"),
        BackupValidationError::UnsupportedVersion(_) => ("unsupported_version", "version"),
        BackupValidationError::DuplicateId { kind: "author", .. } => {
            ("duplicate_author_id", "data.authors")
        }
        BackupValidationError::DuplicateId { kind: "book", .. } => {
            ("duplicate_book_id", "data.books")
        }
        BackupValidationError::InvalidReference(_) => {
            ("missing_author_reference", "data.books[].authorIds")
        }
        BackupValidationError::Malformed(_) => ("malformed_value", "$"),
        BackupValidationError::UnsupportedValue(_) => ("unsupported_value", "$"),
        BackupValidationError::DuplicateId { .. } => ("duplicate_id", "$"),
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

fn validate_snapshot_data(data: &SnapshotBackupDataV1) -> Result<(), BackupValidationError> {
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

    fn snapshot() -> Value {
        json!({
            "format": SNAPSHOT_BACKUP_FORMAT,
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
    fn snapshot_validator_returns_restore_summary() {
        let validation = validate_snapshot_backup(snapshot());
        assert!(validation.response.valid);
        assert!(validation.backup.is_some());
        let summary = validation.response.summary.unwrap();
        assert_eq!(summary.books, 1);
        assert_eq!(summary.authors, 1);
    }

    #[test]
    fn snapshot_validator_reports_header_duplicate_and_reference_errors() {
        let mut invalid_format = snapshot();
        invalid_format["format"] = json!("unknown");
        assert_eq!(
            validate_snapshot_backup(invalid_format).response.errors[0].code,
            "invalid_format"
        );

        let mut invalid_version = snapshot();
        invalid_version["version"] = json!(2);
        assert_eq!(
            validate_snapshot_backup(invalid_version).response.errors[0].code,
            "unsupported_version"
        );

        let mut duplicate = snapshot();
        let author = duplicate["data"]["authors"][0].clone();
        duplicate["data"]["authors"]
            .as_array_mut()
            .unwrap()
            .push(author);
        assert_eq!(
            validate_snapshot_backup(duplicate).response.errors[0].code,
            "duplicate_author_id"
        );

        let mut missing_reference = snapshot();
        missing_reference["data"]["books"][0]["authorIds"] =
            json!(["cccccccc-cccc-4ccc-8ccc-cccccccccccc"]);
        assert_eq!(
            validate_snapshot_backup(missing_reference).response.errors[0].code,
            "missing_author_reference"
        );
    }

    #[test]
    fn snapshot_v1_round_trips_camel_case_contract() {
        let backup = SnapshotBackupV1::parse(snapshot()).unwrap();
        let value = serde_json::to_value(backup).unwrap();
        assert!(value.get("exportedAt").is_some());
        assert!(value["data"]["books"][0].get("authorIds").is_some());
        assert!(value["data"]["books"][0].get("userId").is_none());
    }

    #[test]
    fn rejects_unknown_version_before_deserialization() {
        let mut value = snapshot();
        value["version"] = json!(2);
        assert_eq!(
            SnapshotBackupV1::parse(value).unwrap_err(),
            BackupValidationError::UnsupportedVersion(2)
        );
    }

    #[test]
    fn rejects_duplicate_author_id() {
        let mut value = snapshot();
        let author = value["data"]["authors"][0].clone();
        value["data"]["authors"]
            .as_array_mut()
            .unwrap()
            .push(author);
        assert!(matches!(
            SnapshotBackupV1::parse(value),
            Err(BackupValidationError::DuplicateId { kind: "author", .. })
        ));
    }

    #[test]
    fn rejects_unknown_author_reference() {
        let mut value = snapshot();
        value["data"]["books"][0]["authorIds"] = json!(["cccccccc-cccc-4ccc-8ccc-cccccccccccc"]);
        assert!(matches!(
            SnapshotBackupV1::parse(value),
            Err(BackupValidationError::InvalidReference(_))
        ));
    }
}
