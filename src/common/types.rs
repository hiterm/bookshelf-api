use derive_more::Display;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum BookFormat {
    #[display("eBook")]
    EBook,
    Printed,
    Unknown,
}

impl TryFrom<&str> for BookFormat {
    type Error = ParseBookFormatError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "eBook" => Ok(BookFormat::EBook),
            "Printed" => Ok(BookFormat::Printed),
            "Unknown" => Ok(BookFormat::Unknown),
            _ => Err(ParseBookFormatError(format!(
                "{} is not valid format",
                value
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum BookStore {
    Kindle,
    Unknown,
}

impl TryFrom<&str> for BookStore {
    type Error = ParseBookStoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Kindle" => Ok(BookStore::Kindle),
            "Unknown" => Ok(BookStore::Unknown),
            _ => Err(ParseBookStoreError(format!("{} is not valid store", value))),
        }
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ParseBookFormatError(String);

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ParseBookStoreError(String);

#[cfg(test)]
mod test {
    use crate::common::types::{BookFormat, BookStore};

    #[test]
    fn book_format_ebook_to_string() {
        assert_eq!(BookFormat::EBook.to_string(), "eBook");
    }

    #[test]
    fn book_format_printed_to_string() {
        assert_eq!(BookFormat::Printed.to_string(), "Printed");
    }

    #[test]
    fn book_format_unknown_to_string() {
        assert_eq!(BookFormat::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn book_store_kindle_to_string() {
        assert_eq!(BookStore::Kindle.to_string(), "Kindle");
    }

    #[test]
    fn book_store_unknown_to_string() {
        assert_eq!(BookStore::Unknown.to_string(), "Unknown");
    }
}
