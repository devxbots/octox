use anyhow::Context;
use async_trait::async_trait;
use github_parts::event::Event;
use github_parts::github::{AppId, GitHubHost, PrivateKey};

use octox::{State, Step, Transition, Workflow, WorkflowError};

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
    fn initial_step(&self) -> Box<dyn Step> {
        Box::new(HelloWorldStep)
    }
}

struct HelloWorldStep;

#[async_trait]
impl Step for HelloWorldStep {
    async fn next(self: Box<Self>, state: &mut State) -> Result<Transition, WorkflowError> {
        let event: &Event = state.get().context("failed to get event from state")?;
        Ok(Transition::Complete(format!("received {}", event).into()))
    }
}
