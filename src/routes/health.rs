use axum::http::StatusCode;
use axum::{Extension, Json};
use github_parts::github::token::AppToken;
use reqwest::Client;
use serde::Serialize;

use crate::{AppId, Error, GitHubHost, PrivateKey};

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
    Extension(github_host): Extension<GitHubHost>,
    Extension(app_id): Extension<AppId>,
    Extension(private_key): Extension<PrivateKey>,
) -> (StatusCode, Json<Health>) {
    let mut status_code = StatusCode::OK;

    let github = match check_github(&github_host, &app_id, &private_key).await {
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
    github_host: &GitHubHost,
    app_id: &AppId,
    private_key: &PrivateKey,
) -> Result<(), Error> {
    let endpoint = format!("{}/app", github_host.get());
    let token = match AppToken::new(app_id, private_key) {
        Ok(token) => token,
        Err(error) => {
            return match error {
                github_parts::error::Error::Configuration(_, message) => {
                    Err(Error::Configuration(message))
                }
                _ => Err(Error::Unknown("failed to create GitHub App token".into())),
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
        Err(Error::Unknown(text))
    }
}
