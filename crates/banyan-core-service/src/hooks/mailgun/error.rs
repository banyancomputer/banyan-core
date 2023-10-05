use std::fmt::Display;

use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Debug)]
#[non_exhaustive]
pub struct MailgunHookError {
    kind: MailgunHookErrorKind,
}

impl Display for MailgunHookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.kind))
    }
}

impl std::error::Error for MailgunHookError {}

impl MailgunHookError {
    /// Deault error
    fn default_response(message: Option<String>) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            message.unwrap_or("internal server error".to_string()),
        )
            .into_response()
    }

    pub fn default_error(message: &str) -> Self {
        Self {
            kind: MailgunHookErrorKind::Default(message.to_string()),
        }
    }

    pub fn missing_body() -> Self {
        Self {
            kind: MailgunHookErrorKind::MissingBody,
        }
    }

    pub fn missing_message_id() -> Self {
        Self {
            kind: MailgunHookErrorKind::MissingMessageId,
        }
    }

    pub fn missing_event() -> Self {
        Self {
            kind: MailgunHookErrorKind::MissingEvent,
        }
    }

    pub fn message_does_not_exist() -> Self {
        Self {
            kind: MailgunHookErrorKind::MessageDoesNotExist,
        }
    }

    pub fn invalid_event() -> Self {
        Self {
            kind: MailgunHookErrorKind::InvalidEvent,
        }
    }

    pub fn invalid_message_id() -> Self {
        Self {
            kind: MailgunHookErrorKind::InvalidMessageId,
        }
    }

    pub fn invalid_signature() -> Self {
        Self {
            kind: MailgunHookErrorKind::InvalidSignature,
        }
    }

    pub fn out_of_order_event() -> Self {
        Self {
            kind: MailgunHookErrorKind::OutOfOrderEvent,
        }
    }
}

#[derive(Debug)]
pub enum MailgunHookErrorKind {
    /// Generic 500 Error with message
    Default(String),
    /// Missing Body (406)
    MissingBody,
    /// Missing Event (406)
    MissingEvent,
    /// Missing Message ID (406)
    MissingMessageId,
    /// Invalid Event (406)
    InvalidEvent,
    /// Invalid Message ID (406)
    InvalidMessageId,
    /// Invalid Signature (406)
    InvalidSignature,
    /// Message does not exist in database (406)
    MessageDoesNotExist,
    /// Out of order event (406)
    OutOfOrderEvent,
}

// Mailgun Status Code Behavior
// 200 - OK = Mailgun will determine the webhook POST is successful and not retry.
// 406 - Not Acceptable = Mailgun will determine the POST is rejected and not retry.
// Anything else = Mailgun will retry POSTing according to the schedule below for Webhooks other than the delivery notification.
impl IntoResponse for MailgunHookError {
    fn into_response(self) -> axum::response::Response {
        match self.kind {
            MailgunHookErrorKind::Default(message) => {
                tracing::error!("{message}");
                Self::default_response(Some(message))
            }
            MailgunHookErrorKind::MissingBody => {
                tracing::error!("missing body");
                (StatusCode::NOT_ACCEPTABLE, "missing body".to_string()).into_response()
            }
            MailgunHookErrorKind::MissingEvent => {
                tracing::error!("missing event");
                (StatusCode::NOT_ACCEPTABLE, "missing event".to_string()).into_response()
            }
            MailgunHookErrorKind::MissingMessageId => {
                tracing::error!("missing message id");
                (StatusCode::NOT_ACCEPTABLE, "missing message id".to_string()).into_response()
            }
            MailgunHookErrorKind::InvalidEvent => {
                tracing::error!("invalid event");
                (StatusCode::NOT_ACCEPTABLE, "invalid event".to_string()).into_response()
            }
            MailgunHookErrorKind::InvalidMessageId => {
                tracing::error!("invalid message id");
                (StatusCode::NOT_ACCEPTABLE, "invalid message id".to_string()).into_response()
            }
            MailgunHookErrorKind::InvalidSignature => {
                tracing::error!("invalid signature");
                (StatusCode::NOT_ACCEPTABLE, "invalid signature".to_string()).into_response()
            }
            MailgunHookErrorKind::MessageDoesNotExist => {
                tracing::error!("message does not exist");
                (
                    StatusCode::NOT_ACCEPTABLE,
                    "message does not exist".to_string(),
                )
                    .into_response()
            }
            MailgunHookErrorKind::OutOfOrderEvent => {
                tracing::error!("out of order event");
                (StatusCode::NOT_ACCEPTABLE, "out of order event".to_string()).into_response()
            }
        }
    }
}
