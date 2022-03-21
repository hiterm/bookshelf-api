use thiserror::Error;

#[derive(Debug, Error)]
pub enum PresentationalError {
    #[error(transparent)]
    OtherError(anyhow::Error),
}
