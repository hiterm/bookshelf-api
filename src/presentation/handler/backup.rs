use axum::{
    Extension, Json,
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;
use time::{OffsetDateTime, UtcOffset};

use crate::{
    dependency_injection::BackupExport,
    domain::entity::user::UserId,
    presentation::extractor::claims::Claims,
    use_case::dto::backup::{BackupFullEnvelope, BackupSnapshotEnvelope},
};

pub async fn snapshot_handler(
    claims: Claims,
    Extension(query): Extension<BackupExport>,
) -> Response {
    let user_id = match authenticated_user_id(claims) {
        Some(user_id) => user_id,
        None => return internal_error(),
    };
    let now = OffsetDateTime::now_utc().to_offset(UtcOffset::UTC);
    let data = match query.snapshot(&user_id).await {
        Ok(data) => data,
        Err(error) => {
            tracing::error!(error = ?error, scope = "snapshot", "backup export failed");
            return internal_error();
        }
    };
    backup_response(BackupSnapshotEnvelope::new(now, data))
}

pub async fn full_handler(claims: Claims, Extension(query): Extension<BackupExport>) -> Response {
    let user_id = match authenticated_user_id(claims) {
        Some(user_id) => user_id,
        None => return internal_error(),
    };
    let now = OffsetDateTime::now_utc().to_offset(UtcOffset::UTC);
    let data = match query.full(&user_id).await {
        Ok(data) => data,
        Err(error) => {
            tracing::error!(error = ?error, scope = "full", "backup export failed");
            return internal_error();
        }
    };
    backup_response(BackupFullEnvelope::new(now, data))
}

fn authenticated_user_id(claims: Claims) -> Option<UserId> {
    UserId::new(claims.sub)
        .inspect_err(|error| {
            tracing::error!(error = ?error, "authenticated subject is invalid");
        })
        .ok()
}

fn backup_response<T: Serialize>(envelope: T) -> Response {
    let body = match serde_json::to_vec(&envelope) {
        Ok(body) => body,
        Err(error) => {
            tracing::error!(error = ?error, "backup serialization failed");
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
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "message": "Internal server error" })),
    )
        .into_response()
}
