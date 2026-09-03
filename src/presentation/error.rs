use std::sync::Arc;

use async_graphql::{Error, ErrorExtensions};
use thiserror::Error;

use crate::use_case::error::UseCaseError;

#[derive(Debug, Clone, Error)]
pub enum PresentationalError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error(transparent)]
    OtherError(Arc<anyhow::Error>),
    #[error("{0}")]
    Unexpected(String),
}

impl From<UseCaseError> for PresentationalError {
    fn from(err: UseCaseError) -> Self {
        match err {
            UseCaseError::NotFound {
                entity_type,
                entity_id,
                ..
            } => PresentationalError::NotFound(format!(
                r#"{entity_type} was not found for entity_id "{entity_id}"."#
            )),
            UseCaseError::Validation(_) => PresentationalError::Validation(err.to_string()),
            UseCaseError::Conflict(_) => PresentationalError::Conflict(err.to_string()),
            UseCaseError::Other(_) => {
                PresentationalError::OtherError(Arc::new(anyhow::Error::new(err)))
            }
            UseCaseError::Unexpected(message) => PresentationalError::Unexpected(message),
        }
    }
}

impl ErrorExtensions for PresentationalError {
    fn extend(&self) -> Error {
        let (message, code) = match self {
            PresentationalError::NotFound(message) => (message.as_str(), "NOT_FOUND"),
            PresentationalError::Validation(message) => (message.as_str(), "VALIDATION_ERROR"),
            PresentationalError::Conflict(message) => (message.as_str(), "CONFLICT"),
            PresentationalError::OtherError(error) => {
                tracing::error!(error = ?error, "internal GraphQL error");
                ("Internal server error", "INTERNAL_ERROR")
            }
            PresentationalError::Unexpected(message) => {
                tracing::error!(error = %message, "unexpected GraphQL error");
                ("Internal server error", "INTERNAL_ERROR")
            }
        };

        Error::new(message).extend_with(|_, extensions| extensions.set("code", code))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql::{ErrorExtensions, Value};

    use crate::use_case::error::UseCaseError;

    use super::PresentationalError;

    fn assert_public_error(error: PresentationalError, code: &str) -> async_graphql::Error {
        let error = error.extend();
        assert_eq!(
            error
                .extensions
                .as_ref()
                .and_then(|extensions| extensions.get("code")),
            Some(&Value::from(code))
        );
        error
    }

    #[test]
    fn not_found_omits_user_id_and_has_code() {
        let error = PresentationalError::from(UseCaseError::NotFound {
            entity_type: "book",
            entity_id: "book-id".to_owned(),
            user_id: "secret-user-id".to_owned(),
        });

        let error = assert_public_error(error, "NOT_FOUND");
        assert!(error.message.contains("book-id"));
        assert!(!error.message.contains("secret-user-id"));
    }

    #[test]
    fn validation_has_code_and_keeps_actionable_message() {
        let error = assert_public_error(
            PresentationalError::Validation("invalid title".to_owned()),
            "VALIDATION_ERROR",
        );
        assert!(error.message.contains("invalid title"));
    }

    #[test]
    fn conflict_has_code_and_keeps_actionable_message() {
        let error = assert_public_error(
            PresentationalError::Conflict("name is already in use".to_owned()),
            "CONFLICT",
        );
        assert!(error.message.contains("already in use"));
    }

    #[test]
    fn infrastructure_details_are_hidden() {
        let error = assert_public_error(
            PresentationalError::OtherError(Arc::new(anyhow::anyhow!("database password and SQL"))),
            "INTERNAL_ERROR",
        );
        assert_eq!(error.message, "Internal server error");
        assert!(!error.message.contains("database password"));
    }

    #[test]
    fn unexpected_details_are_hidden() {
        let error = assert_public_error(
            PresentationalError::Unexpected("private invariant detail".to_owned()),
            "INTERNAL_ERROR",
        );
        assert_eq!(error.message, "Internal server error");
        assert!(!error.message.contains("private invariant"));
    }
}
