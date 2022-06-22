use anyhow::Context;
use async_trait::async_trait;
use github_parts::event::Event;
use github_parts::github::app::AppId;
use github_parts::github::{GitHubHost, PrivateKey};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use octox::{Error, Octox, State, Step, Transition, Workflow, WorkflowError};

#[derive(Debug)]
struct HelloWorld;

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

        let body = format!("received {}", event).into();
        println!("{}", &body);

        Ok(Transition::Complete(body))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "hello_world=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let octox = Octox::new().workflow(HelloWorld::constructor)?;
    octox.serve().await
}
