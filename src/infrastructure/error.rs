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

const AUTHOR_NAME_UNIQUE_CONSTRAINT: &str = "author_user_id_name_unique";

/// Maps the author-name uniqueness violation to a conflict and preserves other database errors.
pub(crate) fn classify_author_name_write_error(error: sqlx::Error, name: &str) -> DomainError {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.code().as_deref() == Some("23505")
        && database_error.constraint() == Some(AUTHOR_NAME_UNIQUE_CONSTRAINT)
    {
        return DomainError::AuthorNameConflict(name.to_owned());
    }

    error.into()
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

#[cfg(all(test, feature = "test-with-database"))]
mod database_tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn constraint_classifier_requires_exact_constraint_and_sqlstate(
        pool: PgPool,
    ) -> anyhow::Result<()> {
        sqlx::raw_sql("CREATE SCHEMA classifier; CREATE TABLE classifier.expected (name text CONSTRAINT author_user_id_name_unique UNIQUE); INSERT INTO classifier.expected VALUES ('A'); CREATE TABLE other_unique (name text CONSTRAINT another_unique UNIQUE); INSERT INTO other_unique VALUES ('A'); CREATE TABLE other_fk (name text CONSTRAINT another_fk REFERENCES classifier.expected(name)); CREATE TABLE wrong_state (name text CONSTRAINT author_user_id_name_unique CHECK (name <> 'A'));").execute(&pool).await?;
        for (query, conflict) in [
            ("INSERT INTO classifier.expected VALUES ('A')", true),
            ("INSERT INTO other_unique VALUES ('A')", false),
            ("INSERT INTO other_fk VALUES ('missing')", false),
            ("INSERT INTO wrong_state VALUES ('A')", false),
        ] {
            let sql_error = sqlx::query(query).execute(&pool).await.unwrap_err();
            let classified = classify_author_name_write_error(sql_error, "A");
            if conflict {
                assert!(matches!(classified, DomainError::AuthorNameConflict(name) if name == "A"));
            } else {
                let DomainError::InfrastructureError(cause) = classified else {
                    panic!("unexpected classification: {classified:?}");
                };
                assert!(cause.downcast_ref::<sqlx::Error>().is_some());
            }
        }
        assert!(matches!(
            classify_author_name_write_error(sqlx::Error::RowNotFound, "A"),
            DomainError::InfrastructureError(_)
        ));
        Ok(())
    }
}
