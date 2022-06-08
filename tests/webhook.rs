use std::fs::read;
use std::net::{SocketAddr, TcpListener};

use reqwest::Client;

use octox::{Error, Octox};

use self::workflow::HelloWorld;

mod workflow;

#[tokio::test]
async fn webhook_accepts_valid_signature() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?
        .webhook_secret("secret")?
        .workflow(HelloWorld::constructor)?;

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
        .header("X-GitHub-Event", "not_a_real_event")
        .header(
            "X-Hub-Signature-256",
            "sha256=ba9f77aa6bc9740e9be7f68e4e21a64821cc5b59fd286d409d605a0b8affe7ff",
        )
        .body(body)
        .send()
        .await?;

    assert_eq!(
        "\"received unsupported event\"",
        response.text().await.unwrap()
    );
    Ok(())
}

#[tokio::test]
async fn webhook_requires_signature() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?
        .webhook_secret("secret")?
        .workflow(HelloWorld::constructor)?;

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

    assert_eq!(400, response.status().as_u16());

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

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    let octox = Octox::new()
        .tcp_listener(listener)?
        .github_host(mockito::server_url())?
        .webhook_secret("secret")?
        .workflow(HelloWorld::constructor)?;

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

    assert_eq!(401, response.status().as_u16());

    assert!(response
        .text()
        .await
        .unwrap()
        .contains("X-Hub-Signature-256 header is invalid"));
    Ok(())
}
