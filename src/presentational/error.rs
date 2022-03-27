use thiserror::Error;

use crate::use_case::error::UseCaseError;

#[derive(Debug, Error)]
pub enum PresentationalError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    OtherError(anyhow::Error),
}

impl From<UseCaseError> for PresentationalError {
    fn from(err: UseCaseError) -> Self {
        match err {
            UseCaseError::NotFound { .. } => PresentationalError::NotFound(err.to_string()),
            UseCaseError::Validation(_) => PresentationalError::Validation(err.to_string()),
            UseCaseError::Other(_) => PresentationalError::OtherError(anyhow::Error::new(err)),
        }
    }
}
