use std::sync::Arc;

use axum::body::Bytes;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use github_parts::event::Event;
use serde_json::Value;

use crate::auth::{verify_signature, AuthError};
use crate::error::Error;
use crate::github::WebhookSecret;
use crate::workflow::Workflow;

#[tracing::instrument(skip(body))]
pub async fn webhook(
    headers: HeaderMap,
    body: Bytes,
    Extension(webhook_secret): Extension<WebhookSecret>,
    Extension(workflow): Extension<Arc<dyn Workflow>>,
) -> Result<Json<Value>, Error> {
    let signature = get_signature(&headers)?;
    verify_signature(&body, &signature, &webhook_secret)?;

    let event_type = get_event(&headers)?;
    let event = deserialize_event(&event_type, &body)?;

    let body = workflow.process(event).await?;

    Ok(Json(body))
}

#[tracing::instrument]
fn get_signature(headers: &HeaderMap) -> Result<String, AuthError> {
    get_header(headers, "X-Hub-Signature-256")
}

#[tracing::instrument]
fn get_event(headers: &HeaderMap) -> Result<String, AuthError> {
    get_header(headers, "X-GitHub-Event")
}

#[tracing::instrument]
fn get_header(headers: &HeaderMap, header: &str) -> Result<String, AuthError> {
    headers
        .get(header)
        .and_then(|header| header.to_str().ok())
        .map(String::from)
        .ok_or_else(|| AuthError::MissingHeader(header.into()))
}

#[tracing::instrument(skip(body))]
fn deserialize_event(event_type: &str, body: &Bytes) -> Result<Event, Error> {
    let event = match event_type {
        "check_run" => Event::CheckRun(serde_json::from_slice(body)?),
        _ => Event::Unsupported(serde_json::from_slice(body)?),
    };

    Ok(event)
}
