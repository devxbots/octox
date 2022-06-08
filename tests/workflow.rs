use async_trait::async_trait;
use github_parts::event::Event;
use serde_json::Value;

use octox::{AppId, GitHubHost, PrivateKey, Workflow, WorkflowError};

#[derive(Debug)]
pub struct HelloWorld;

impl HelloWorld {
    pub fn constructor(
        _github_host: GitHubHost,
        _app_id: AppId,
        _private_key: PrivateKey,
    ) -> Box<dyn Workflow> {
        Box::new(HelloWorld)
    }
}

#[async_trait]
impl Workflow for HelloWorld {
    async fn process(&self, event: Event) -> Result<Value, WorkflowError> {
        Ok(format!("received {}", event).into())
    }
}
