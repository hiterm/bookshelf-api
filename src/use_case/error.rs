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

#[cfg(test)]
mod tests {
    use crate::domain::error::DomainError;

    use super::UseCaseError;

    #[test]
    fn domain_validation_error_becomes_use_case_validation_error() {
        let domain_err = DomainError::Validation("invalid input".to_string());
        let use_case_err = UseCaseError::from(domain_err);
        assert!(matches!(use_case_err, UseCaseError::Validation(_)));
    }

    #[test]
    fn domain_not_found_error_becomes_use_case_not_found_error() {
        let domain_err = DomainError::NotFound {
            entity_type: "book",
            entity_id: "123".to_string(),
            user_id: "user1".to_string(),
        };
        let use_case_err = UseCaseError::from(domain_err);
        assert!(matches!(use_case_err, UseCaseError::NotFound { .. }));
    }

    #[test]
    fn domain_infrastructure_error_becomes_use_case_other_error() {
        let domain_err = DomainError::InfrastructureError(anyhow::anyhow!("db error"));
        let use_case_err = UseCaseError::from(domain_err);
        assert!(matches!(use_case_err, UseCaseError::Other(_)));
    }

    #[test]
    fn domain_unexpected_error_becomes_use_case_unexpected_error() {
        let domain_err = DomainError::Unexpected("something went wrong".to_string());
        let use_case_err = UseCaseError::from(domain_err);
        assert!(matches!(use_case_err, UseCaseError::Unexpected(_)));
    }
}
