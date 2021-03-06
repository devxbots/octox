use std::sync::Arc;

use axum::http::StatusCode;
use axum::{Extension, Json};
use github_parts::github::token::TokenFactory;
use parking_lot::Mutex;
use reqwest::Client;
use serde::Serialize;

use crate::{Error, GitHubHost, SharedTokenFactory};

#[derive(Debug, Serialize)]
pub struct Health {
    github: String,
}

#[derive(Debug, Serialize)]
struct Claims {
    iat: i64,
    iss: String,
    exp: i64,
}

#[tracing::instrument]
pub async fn health(
    Extension(token_factory): Extension<SharedTokenFactory>,
    Extension(github_host): Extension<GitHubHost>,
) -> (StatusCode, Json<Health>) {
    let mut status_code = StatusCode::OK;

    let github = match check_github(&token_factory, &github_host).await {
        Ok(_) => "ok".into(),
        Err(error) => {
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
            error.to_string()
        }
    };

    (status_code, Json(Health { github }))
}

#[tracing::instrument]
async fn check_github(
    token_factory: &Arc<Mutex<TokenFactory>>,
    github_host: &GitHubHost,
) -> Result<(), Error> {
    let endpoint = format!("{}/app", github_host.get());

    let token = match token_factory.lock().app() {
        Ok(token) => token,
        Err(error) => {
            return match error {
                github_parts::error::Error::Configuration(_, message) => {
                    Err(Error::Configuration(message))
                }
                _ => Err(Error::UnexpectedError(error.into())),
            }
        }
    };

    let response = Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", token.get()))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "devxbots/octox")
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let text = response.text().await?;
        Err(Error::UnexpectedError(anyhow::Error::msg(text)))
    }
}
