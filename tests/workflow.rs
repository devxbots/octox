use github_parts::event::Event;
use serde_json::Value;

use octox::{Workflow, WorkflowError};

#[derive(Debug)]
pub struct HelloWorld;

impl Workflow for HelloWorld {
    fn process(&self, event: Event) -> Result<Value, WorkflowError> {
        Ok(format!("received {}", event).into())
    }
}
