use thiserror::Error;

#[derive(Debug, Error)]
pub enum PresentationalError {
    #[error(transparent)]
    OtherError(anyhow::Error),
}

impl From<uuid::Error> for PresentationalError {
    fn from(error: uuid::Error) -> Self {
        PresentationalError::OtherError(anyhow::Error::new(error))
    }
}
