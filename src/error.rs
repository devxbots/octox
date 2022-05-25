use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to initialize the web framework")]
    Axum(#[from] hyper::Error),
    #[error("{0}")]
    Configuration(String),
    #[error("failed to initialize resources")]
    Io(#[from] std::io::Error),
    #[error("an unknown error occurred")]
    Unknown(String),
}
