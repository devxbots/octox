use std::sync::Arc;

use github_parts::event::Event;
use serde_json::Value;

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

    let octox = Octox::new().workflow(Arc::new(HelloWorld))?;
    octox.serve().await
}
