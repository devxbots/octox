use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::auth::AuthError;
use crate::workflow::WorkflowError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Configuration(String),

    #[error(transparent)]
    Client(#[from] AuthError),

    #[error(transparent)]
    ExternalResource(#[from] reqwest::Error),

    #[error("failed to deserialize incoming request payload")]
    Payload(#[from] serde_json::Error),

    #[error(transparent)]
    Workflow(#[from] WorkflowError),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Client(error) => error.into_response(),
            _ => {
                let body = self.to_string();
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
        }
    }
}
