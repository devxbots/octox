use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to call external API")]
    Api(#[from] reqwest::Error),
    #[error("failed to initialize the web framework")]
    Axum(#[from] hyper::Error),
    #[error("{0}")]
    Configuration(String),
    #[error("failed to initialize resource")]
    Io(#[from] std::io::Error),
    #[error("failed to create JWT")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("{0}")]
    Unknown(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = self.to_string();

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
