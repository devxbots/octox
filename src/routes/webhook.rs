use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::Extension;

use crate::auth::{verify_signature, AuthError};
use crate::WebhookSecret;

pub async fn webhook(
    headers: HeaderMap,
    body: Bytes,
    Extension(webhook_secret): Extension<WebhookSecret>,
) -> Result<StatusCode, AuthError> {
    let signature = get_signature(headers)?;
    verify_signature(&body, &signature, &webhook_secret)?;

    Ok(StatusCode::OK)
}

fn get_signature(headers: HeaderMap) -> Result<String, AuthError> {
    get_header(headers, "X-Hub-Signature-256")
}

fn get_header(headers: HeaderMap, header: &str) -> Result<String, AuthError> {
    headers
        .get(header)
        .and_then(|header| header.to_str().ok())
        .map(String::from)
        .ok_or_else(|| AuthError::MissingHeader(header.into()))
}
