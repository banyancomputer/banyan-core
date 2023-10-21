use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum MailgunHookError {
    #[error("failed to decode signature")]
    FailedToDecodeSignature(hex::FromHexError),

    #[error("invalid signature")]
    InvalidSignature(ring::error::Unspecified),
}

impl IntoResponse for MailgunHookError {
    fn into_response(self) -> Response {
        let err_msg = serde_json::json!({ "msg": self.to_string() });
        (StatusCode::NOT_ACCEPTABLE, Json(err_msg)).into_response()
    }
}
