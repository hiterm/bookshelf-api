use thiserror::Error;
use validator::ValidationErrors;

use crate::common::types::{ParseBookFormatError, ParseBookStoreError};

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("{0}")]
    Validation(String),
    #[error(r#"{entity_type} was not found for entity_id "{entity_id}" and user_id "{user_id}"."#)]
    NotFound {
        entity_type: &'static str,
        entity_id: String,
        user_id: String,
    },
    #[error(transparent)]
    InfrastructureError(anyhow::Error),
    #[error("{0}")]
    Unexpected(String),
}

impl From<ValidationErrors> for DomainError {
    fn from(err: ValidationErrors) -> Self {
        DomainError::Validation(err.to_string())
    }
}

impl From<ParseBookStoreError> for DomainError {
    fn from(err: ParseBookStoreError) -> Self {
        DomainError::Validation(err.to_string())
    }
}

impl From<ParseBookFormatError> for DomainError {
    fn from(err: ParseBookFormatError) -> Self {
        DomainError::Validation(err.to_string())
    }
}
