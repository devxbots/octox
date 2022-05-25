use std::net::{SocketAddr, TcpListener};

use reqwest::Client;

use octox::{Error, Octox};

#[tokio::test]
async fn root_returns_ok() -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
    let addr = listener.local_addr()?;

    let octox = Octox::new().listener(listener)?;

    tokio::spawn(async move {
        octox.serve().await.unwrap();
    });

    let response = Client::new()
        .get(format!("http://{}", addr))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());

    Ok(())
}

#[tokio::test]
async fn root_prints_hello_world() -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
    let addr = listener.local_addr()?;

    let octox = Octox::new().listener(listener)?;

    tokio::spawn(async move {
        octox.serve().await.unwrap();
    });

    let response = Client::new()
        .get(format!("http://{}", addr))
        .send()
        .await
        .expect("failed to execute request");

    let body = response.text().await.unwrap();

    assert_eq!("Hello, World!", body);

    Ok(())
}
