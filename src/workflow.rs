use std::fmt::Debug;

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use github_parts::event::Event;
use thiserror::Error;

#[async_trait]
pub trait Workflow: Debug + Sync + Send {
    async fn process(&self, event: Event) -> Result<serde_json::Value, WorkflowError>;
}

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("configuration was not valid")]
    Configuration,

    #[error("{0}")]
    MissingData(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for WorkflowError {
    fn into_response(self) -> Response {
        match self {
            WorkflowError::Configuration => StatusCode::OK.into_response(),
            WorkflowError::MissingData(error) => (StatusCode::BAD_REQUEST, error).into_response(),
            WorkflowError::UnexpectedError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
        }
    }
}
