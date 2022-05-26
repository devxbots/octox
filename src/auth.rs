use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use thiserror::Error;

use crate::WebhookSecret;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Error)]
pub enum AuthError {
    #[error("missing {0} header")]
    MissingHeader(String),
    #[error("failed to initialize cryptographic key")]
    FailedHmacInitialization,
    #[error("X-Hub-Signature-256 header has the wrong format")]
    WrongSignatureFormat,
    #[error("failed to decode the X-Hub-Signature-256 header")]
    FailedDecodingSignature,
    #[error("X-Hub-Signature-256 header is invalid")]
    InvalidSignature,
    #[error("failed to deserialize the body based on the X-GitHub-Event header")]
    UnexpectedPayload,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let message = self.to_string();

        let status = match self {
            AuthError::MissingHeader(_) => StatusCode::BAD_REQUEST,
            AuthError::FailedHmacInitialization => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::WrongSignatureFormat => StatusCode::BAD_REQUEST,
            AuthError::FailedDecodingSignature => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidSignature => StatusCode::UNAUTHORIZED,
            AuthError::UnexpectedPayload => StatusCode::BAD_REQUEST,
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub fn verify_signature(
    body: &Bytes,
    signature: &str,
    secret: &WebhookSecret,
) -> Result<(), AuthError> {
    let mut hmac = match HmacSha256::new_from_slice(secret.get().as_bytes()) {
        Ok(hmac) => hmac,
        Err(_) => return Err(AuthError::FailedHmacInitialization),
    };
    hmac.update(body);

    let signature = match signature.split('=').last() {
        Some(signature) => signature,
        None => return Err(AuthError::WrongSignatureFormat),
    };

    let decoded_signature = match hex::decode(signature) {
        Ok(signature) => signature,
        Err(_) => return Err(AuthError::FailedDecodingSignature),
    };

    if hmac.verify_slice(decoded_signature.as_slice()).is_err() {
        return Err(AuthError::InvalidSignature);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::body::Bytes;

    use crate::WebhookSecret;

    use super::verify_signature;

    #[test]
    fn verify_signature_with_valid_signature() {
        let body = "verify_signature";
        let signature = "sha256=22568b39613009e6d1b1fd063085c05063998bda5243a597c0cc524e044990ae";
        let secret = WebhookSecret::new("verify_signature".into());

        assert!(verify_signature(&Bytes::from(body), signature, &secret).is_ok());
    }

    #[test]
    fn verify_signature_with_empty_body() {
        let body = "";
        let signature = "sha256=22568b39613009e6d1b1fd063085c05063998bda5243a597c0cc524e044990ae";
        let secret = WebhookSecret::new("verify_signature".into());

        assert!(verify_signature(&Bytes::from(body), signature, &secret).is_err());
    }

    #[test]
    fn verify_signature_with_empty_signature() {
        let body = "verify_signature";
        let signature = "";
        let secret = WebhookSecret::new("verify_signature".into());

        assert!(verify_signature(&Bytes::from(body), signature, &secret).is_err());
    }

    #[test]
    fn verify_signature_with_empty_body_secret_and_signature() {
        let body = "";
        let signature = "";
        let secret = WebhookSecret::new("".into());

        assert!(verify_signature(&Bytes::from(body), signature, &secret).is_err());
    }
}
