use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error(transparent)]
    InfrastructureError(anyhow::Error),
}
