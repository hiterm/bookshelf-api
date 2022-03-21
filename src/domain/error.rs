use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    InfrastructureError(anyhow::Error),
}
