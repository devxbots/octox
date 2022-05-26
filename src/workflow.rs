use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt::Debug;

use github_parts::event::Event;
use thiserror::Error;

pub trait Workflow: Debug + Sync + Send {
    fn process(&self, event: Event) -> Result<serde_json::Value, WorkflowError>;
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Error)]
pub enum WorkflowError {
    #[error("{0}")]
    Unknown(String),
}

impl IntoResponse for WorkflowError {
    fn into_response(self) -> Response {
        match self {
            WorkflowError::Unknown(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
            }
        }
    }
}
