use std::sync::Arc;

use async_graphql::{ErrorExtensionValues, ErrorExtensions};
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
            UseCaseError::NotFound { .. } => PresentationalError::NotFound(err.to_string()),
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
    fn extend(&self) -> async_graphql::Error {
        let mut error = async_graphql::Error::new(self.to_string());
        error.extensions = Some(ErrorExtensionValues::default());
        error = error.extend_with(|_, extensions| {
            extensions.set(
                "code",
                match self {
                    PresentationalError::NotFound(_) => "NOT_FOUND",
                    PresentationalError::Validation(_) => "VALIDATION_ERROR",
                    PresentationalError::Conflict(_) => "CONFLICT",
                    PresentationalError::OtherError(_) => "OTHER_ERROR",
                    PresentationalError::Unexpected(_) => "UNEXPECTED_ERROR",
                },
            );
        });
        error
    }
}
