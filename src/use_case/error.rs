use thiserror::Error;

use crate::domain::error::DomainError;

#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error(r#"{entity_type} was not found for entity_id "{entity_id}" and user_id "{user_id}"."#)]
    NotFound {
        entity_type: &'static str,
        entity_id: String,
        user_id: String,
    },
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Other(anyhow::Error),
}

impl From<DomainError> for UseCaseError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Validation(message) => UseCaseError::Validation(message),
            DomainError::InfrastructureError(cause) => UseCaseError::Other(cause),
        }
    }
}
