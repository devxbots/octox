use reqwest::Client;
use std::fs::read;
use std::net::{SocketAddr, TcpListener};

use octox::{Error, Octox};

#[tokio::test]
async fn webhook_accepts_valid_signature() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?
        .webhook_secret("secret")?;

    tokio::spawn(async move {
        octox.serve().await.unwrap();
    });

    let fixture = format!(
        "{}/tests/fixtures/check_run.created.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let body = read(fixture).unwrap();

    let response = Client::new()
        .post(format!("http://{}/", addr))
        .header(
            "X-Hub-Signature-256",
            "sha256=0ee69dc1afb2d6fd5d09d0163b36c228c3db01dfec1f31c59944938a0bfb4502",
        )
        .body(body)
        .send()
        .await?;

    assert_eq!("", response.text().await.unwrap());
    Ok(())
}

#[tokio::test]
async fn webhook_requires_signature() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?
        .webhook_secret("secret")?;

    tokio::spawn(async move {
        octox.serve().await.unwrap();
    });

    let fixture = format!(
        "{}/tests/fixtures/check_run.created.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let body = read(fixture).unwrap();

    let response = Client::new()
        .post(format!("http://{}/", addr))
        .body(body)
        .send()
        .await?;

    assert!(response
        .text()
        .await
        .unwrap()
        .contains("missing X-Hub-Signature-256 header"));
    Ok(())
}

#[tokio::test]
async fn webhook_rejects_invalid_signature() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())?;
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?
        .webhook_secret("secret")?;

    tokio::spawn(async move {
        octox.serve().await.unwrap();
    });

    let fixture = format!(
        "{}/tests/fixtures/check_run.created.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let body = read(fixture).unwrap();

    let response = Client::new()
        .post(format!("http://{}/", addr))
        .header(
            "X-Hub-Signature-256",
            "sha256=21fc0cdd18aa13806dec49fa657a57571704de8690eaeda53c103493d55d6a37",
        )
        .body(body)
        .send()
        .await?;

    assert!(response
        .text()
        .await
        .unwrap()
        .contains("X-Hub-Signature-256 header is invalid"));
    Ok(())
}
