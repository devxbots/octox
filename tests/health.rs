use mockito::mock;
use std::net::{SocketAddr, TcpListener};

use reqwest::Client;

use octox::{Error, Octox};

#[tokio::test]
async fn health_returns_ok() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let _mock = mock("GET", "/app").with_status(200).create();

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?
        .app_id(1)?
        .private_key(include_str!("fixtures/private-key.pem"))?;

    tokio::spawn(async move {
        octox.serve().await.unwrap();
    });

    let response = Client::new()
        .get(format!("http://{}/health", addr))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());

    Ok(())
}

#[tokio::test]
async fn health_returns_error() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let _mock = mock("GET", "/app").with_status(401).create();

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?;

    tokio::spawn(async move {
        octox.serve().await.unwrap();
    });

    let response = Client::new()
        .get(format!("http://{}/health", addr))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_server_error());

    Ok(())
}
