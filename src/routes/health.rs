use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
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
    let token = new_app_token(app_id, private_key)?;

    let response = Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", token))
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

#[tracing::instrument]
fn new_app_token(app_id: &AppId, private_key: &PrivateKey) -> Result<String, Error> {
    let now = Utc::now();

    let issued_at = now
        .checked_sub_signed(Duration::seconds(60))
        .ok_or_else(|| Error::Unknown("failed to calculate issued_at date for JWT".into()))?;

    let expires_at = now
        .checked_add_signed(Duration::minutes(10))
        .ok_or_else(|| Error::Unknown("failed to calculate expires_at date for JWT".into()))?;

    let claims = Claims {
        iat: issued_at.timestamp(),
        iss: app_id.get().to_string(),
        exp: expires_at.timestamp(),
    };

    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(private_key.get().as_bytes())?;

    encode(&header, &claims, &key).map_err(|err| err.into())
}
