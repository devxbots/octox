use std::sync::Arc;

use github_parts::event::Event;
use serde_json::Value;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use octox::{Error, Octox, Workflow, WorkflowError};

#[derive(Debug)]
struct HelloWorld;

impl Workflow for HelloWorld {
    fn process(&self, event: Event) -> Result<Value, WorkflowError> {
        let body = format!("received {}", event).into();

        println!("{}", &body);

        Ok(body)
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

    let octox = Octox::new().workflow(Arc::new(HelloWorld))?;
    octox.serve().await
}
