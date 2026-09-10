use axum::{
    Extension, Json,
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

use crate::{
    dependency_injection::BackupQuery,
    domain::entity::user::UserId,
    presentation::extractor::claims::Claims,
    use_case::dto::backup::{BackupEnvelope, BackupScope},
};

pub async fn snapshot_handler(
    claims: Claims,
    Extension(query): Extension<BackupQuery>,
) -> Response {
    export(claims, query, BackupScope::Snapshot).await
}

pub async fn full_handler(claims: Claims, Extension(query): Extension<BackupQuery>) -> Response {
    export(claims, query, BackupScope::Full).await
}

async fn export(claims: Claims, query: BackupQuery, scope: BackupScope) -> Response {
    let user_id = match UserId::new(claims.sub) {
        Ok(user_id) => user_id,
        Err(error) => {
            tracing::error!(error = ?error, "authenticated subject is invalid");
            return internal_error();
        }
    };
    let now = OffsetDateTime::now_utc().to_offset(UtcOffset::UTC);
    let data = match query.export(&user_id, scope).await {
        Ok(data) => data,
        Err(error) => {
            tracing::error!(error = ?error, scope = scope.as_str(), "backup export failed");
            return internal_error();
        }
    };
    let exported_at = match now.format(&Rfc3339) {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(error = ?error, "backup timestamp formatting failed");
            return internal_error();
        }
    };
    let body = match serde_json::to_vec(&BackupEnvelope::new(scope, exported_at, data)) {
        Ok(body) => body,
        Err(error) => {
            tracing::error!(error = ?error, "backup serialization failed");
            return internal_error();
        }
    };
    let filename = format!(
        "bookshelf-backup-{}-{:04}-{:02}-{:02}T{:02}{:02}{:02}Z.json",
        scope.as_str(),
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    let disposition = match HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(error = ?error, "backup response header formatting failed");
            return internal_error();
        }
    };
    let mut response = Response::new(Body::from(body));
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, disposition);
    response
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "message": "Internal server error" })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use crate::use_case::dto::backup::{BackupData, BackupEnvelope, BackupScope};

    #[test]
    fn envelope_uses_versioned_camel_case_contract() {
        let value = serde_json::to_value(BackupEnvelope::new(
            BackupScope::Snapshot,
            "2026-09-11T02:00:00Z".to_owned(),
            BackupData {
                authors: vec![],
                books: vec![],
                history: None,
            },
        ))
        .expect("serializable backup");

        assert_eq!(value["format"], "bookshelf-backup");
        assert_eq!(value["version"], 1);
        assert_eq!(value["scope"], "snapshot");
        assert_eq!(value["exportedAt"], "2026-09-11T02:00:00Z");
        assert!(value["data"].get("history").is_none());
        assert!(!value.to_string().contains("user_id"));
    }
}
