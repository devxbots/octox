use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::Extension;
use github_parts::event::Event;

use crate::auth::{verify_signature, AuthError};
use crate::WebhookSecret;

pub async fn webhook(
    headers: HeaderMap,
    body: Bytes,
    Extension(webhook_secret): Extension<WebhookSecret>,
) -> Result<StatusCode, AuthError> {
    let signature = get_signature(&headers)?;
    verify_signature(&body, &signature, &webhook_secret)?;

    let event_type = get_event(&headers)?;
    let _event = deserialize_event(&event_type, &body)?;

    Ok(StatusCode::OK)
}

fn get_signature(headers: &HeaderMap) -> Result<String, AuthError> {
    get_header(headers, "X-Hub-Signature-256")
}

fn get_event(headers: &HeaderMap) -> Result<String, AuthError> {
    get_header(headers, "X-GitHub-Event")
}

fn get_header(headers: &HeaderMap, header: &str) -> Result<String, AuthError> {
    headers
        .get(header)
        .and_then(|header| header.to_str().ok())
        .map(String::from)
        .ok_or_else(|| AuthError::MissingHeader(header.into()))
}

fn deserialize_event(_event_type: &str, body: &Bytes) -> Result<Event, AuthError> {
    let event =
        Event::Unsupported(serde_json::from_slice(body).map_err(|_| AuthError::UnexpectedPayload)?);

    Ok(event)
}
