use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    // Generic internal error
    #[error("internal error")]
    InternalError(#[from] anyhow::Error),
    #[error("failed to authenticate")]
    RequestError(#[from] reqwest::Error),
    #[error("Unable to read file or start subprocess")]
    UnableToRead(#[from] std::io::Error),
    #[error("failed to serve image")]
    NotFound,
    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity::Error),
}