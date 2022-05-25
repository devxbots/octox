use octox::{Error, Octox};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let octox = Octox::new();
    octox.serve().await
}
