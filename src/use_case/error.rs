use thiserror::Error;

use crate::domain::error::DomainError;

#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error("{0}")]
    Validation(String),
    #[error(r#"{entity_type} was not found for entity_id "{entity_id}" and user_id "{user_id}"."#)]
    NotFound {
        entity_type: &'static str,
        entity_id: String,
        user_id: String,
    },
    #[error(transparent)]
    Other(anyhow::Error),
    #[error("{0}")]
    Unexpected(String),
}

impl From<DomainError> for UseCaseError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Validation(message) => UseCaseError::Validation(message),
            DomainError::NotFound {
                entity_type,
                entity_id,
                user_id,
            } => UseCaseError::NotFound {
                entity_type,
                entity_id,
                user_id,
            },
            DomainError::InfrastructureError(_) => UseCaseError::Other(anyhow::Error::new(err)),
            DomainError::Unexpected(message) => UseCaseError::Unexpected(message),
        }
    }
}
