use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    InfrastructureError(anyhow::Error),
}

impl From<ValidationErrors> for DomainError {
    fn from(err: ValidationErrors) -> Self {
        DomainError::Validation(err.to_string())
    }
}
