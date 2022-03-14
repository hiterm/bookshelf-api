use crate::domain::error::domain_error::DomainError;

impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        DomainError::InfrastructureError(anyhow::Error::new(error))
    }
}
