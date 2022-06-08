use async_trait::async_trait;
use github_parts::event::Event;
use serde_json::Value;

use octox::{Workflow, WorkflowError};

#[derive(Debug)]
pub struct HelloWorld;

#[async_trait]
impl Workflow for HelloWorld {
    async fn process(&self, event: Event) -> Result<Value, WorkflowError> {
        Ok(format!("received {}", event).into())
    }
}
