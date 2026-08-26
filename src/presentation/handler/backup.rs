use axum::{
    Extension, Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    presentation::extractor::claims::Claims,
    use_case::{error::UseCaseError, traits::backup::BackupUseCase},
};

#[derive(Debug)]
pub struct BackupHttpError(UseCaseError);

impl From<UseCaseError> for BackupHttpError {
    fn from(value: UseCaseError) -> Self {
        Self(value)
    }
}

impl IntoResponse for BackupHttpError {
    fn into_response(self) -> Response {
        match self.0 {
            UseCaseError::BackupValidation(response) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(response)).into_response()
            }
            error => {
                let (status, code) = match error {
                    UseCaseError::Validation(_) => {
                        (StatusCode::UNPROCESSABLE_ENTITY, "invalid_backup")
                    }
                    UseCaseError::Conflict(_) => (StatusCode::CONFLICT, "restore_conflict"),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
                };
                (
                    status,
                    Json(json!({"error": code, "message": error.to_string()})),
                )
                    .into_response()
            }
        }
    }
}

pub async fn validate_snapshot<BU: BackupUseCase>(
    claims: Claims,
    Extension(use_case): Extension<BU>,
    Json(value): Json<Value>,
) -> Result<Json<impl Serialize>, BackupHttpError> {
    Ok(Json(use_case.validate_snapshot(&claims.sub, value).await?))
}

pub async fn export_snapshot<BU: BackupUseCase>(
    claims: Claims,
    Extension(use_case): Extension<BU>,
) -> Result<Json<impl Serialize>, BackupHttpError> {
    Ok(Json(use_case.export_snapshot(&claims.sub).await?))
}

pub async fn export_full<BU: BackupUseCase>(
    claims: Claims,
    Extension(use_case): Extension<BU>,
) -> Result<Json<impl Serialize>, BackupHttpError> {
    Ok(Json(use_case.export_full(&claims.sub).await?))
}

pub async fn restore_snapshot<BU: BackupUseCase>(
    claims: Claims,
    Extension(use_case): Extension<BU>,
    Json(value): Json<Value>,
) -> Result<StatusCode, BackupHttpError> {
    use_case.restore_snapshot(&claims.sub, value).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_maps_to_client_error() {
        let response =
            BackupHttpError(UseCaseError::Validation("bad backup".to_string())).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn conflict_maps_to_conflict() {
        let response = BackupHttpError(UseCaseError::Conflict("busy".to_string())).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
