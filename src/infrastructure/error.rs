use crate::{
    common::types::{BookFormat, BookStore},
    domain::error::DomainError,
};

impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        DomainError::InfrastructureError(anyhow::Error::new(error))
    }
}

/// Reclassifies validation of stored data without changing client input validation.
pub(crate) fn persisted_value<T>(
    field: &'static str,
    result: Result<T, DomainError>,
) -> Result<T, DomainError> {
    result.map_err(|error| match error {
        DomainError::Validation(message) => {
            DomainError::Unexpected(format!("invalid persisted {field}: {message}"))
        }
        other => other,
    })
}

pub(crate) fn persisted_book_format(value: &str) -> Result<BookFormat, DomainError> {
    persisted_value(
        "book format",
        BookFormat::try_from(value).map_err(DomainError::from),
    )
}

pub(crate) fn persisted_book_store(value: &str) -> Result<BookStore, DomainError> {
    persisted_value(
        "book store",
        BookStore::try_from(value).map_err(DomainError::from),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::book::Priority;

    #[test]
    fn persisted_enums_accept_all_existing_values() {
        for value in ["eBook", "Printed", "Unknown"] {
            assert_eq!(
                persisted_book_format(value).unwrap(),
                BookFormat::try_from(value).unwrap()
            );
        }
        for value in ["Kindle", "Unknown"] {
            assert_eq!(
                persisted_book_store(value).unwrap(),
                BookStore::try_from(value).unwrap()
            );
        }
    }

    #[test]
    fn invalid_persisted_enums_are_internal_errors() {
        assert!(matches!(
            persisted_book_format("private-format"),
            Err(DomainError::Unexpected(_))
        ));
        assert!(matches!(
            persisted_book_store("private-store"),
            Err(DomainError::Unexpected(_))
        ));
    }

    #[test]
    fn persisted_validation_does_not_change_input_validation() {
        assert!(matches!(Priority::new(-1), Err(DomainError::Validation(_))));
        assert!(matches!(
            persisted_value("book priority", Priority::new(-1)),
            Err(DomainError::Unexpected(_))
        ));
        let error =
            persisted_value::<()>("field", Err(sqlx::Error::RowNotFound.into())).unwrap_err();
        assert!(matches!(error, DomainError::InfrastructureError(_)));
    }
}
