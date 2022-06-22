use std::fmt::Debug;

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use github_parts::event::Event;
use thiserror::Error;

use crate::State;

#[async_trait]
pub trait Workflow: Debug + Sync + Send {
    fn initial_state(&self) -> State {
        State::new()
    }

    fn initial_step(&self) -> Box<dyn Step>;

    async fn execute(&self, event: Event) -> Result<serde_json::Value, WorkflowError> {
        let mut step = self.initial_step();
        let mut state = self.initial_state();

        state.insert(event);

        loop {
            step = match step.next(&mut state).await? {
                Transition::Next(step) => step,
                Transition::Complete(result) => return Ok(result),
            }
        }
    }
}

#[async_trait]
pub trait Step: Send + Sync {
    async fn next(self: Box<Self>, state: &mut State) -> Result<Transition, WorkflowError>;
}

pub enum Transition {
    Next(Box<dyn Step>),
    Complete(serde_json::Value),
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
