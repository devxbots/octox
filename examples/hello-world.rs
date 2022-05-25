use octox::{Error, Octox};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let octox = Octox::new();
    octox.serve().await
}
