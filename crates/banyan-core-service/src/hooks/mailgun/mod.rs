mod event_data;
mod mailgun_event;
mod mailgun_hook_request;
mod signature;
mod user_variables;

pub(crate) use event_data::EventData;
pub(crate) use mailgun_event::MailgunEvent;
pub(crate) use mailgun_hook_request::MailgunHookRequest;
pub(crate) use signature::Signature;
pub(crate) use user_variables::UserVariables;

use axum::Json;
use axum::http::StatusCode;
use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;
use crate::database::Database;
use crate::database::models::EmailMessageState;

pub async fn handler(
    State(state): State<AppState>,
    Json(request): Json<MailgunHookRequest>,
) -> Result<Response, MailgunHookError> {
    let mailgun_webhook_key = match state.secrets().mailgun_signing_key() {
        Some(mwk) => mwk,
        None => return Err(MailgunHookError::NotConfigured),
    };

    request.verify_signature(&mailgun_webhook_key)?;

    let message_id = request.message_id().to_string();
    let reported_state = request.event();

    let database = state.database();

    let email_state = sqlx::query_as!(
        EmailState,
        "SELECT account_id, state as 'state: EmailMessageState' FROM emails WHERE id = $1;",
        message_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(MailgunHookError::QueryFailed)?;

    todo!()
}

#[derive(sqlx::FromRow)]
struct EmailState {
    account_id: String,
    state: EmailMessageState,
}

#[derive(Debug, thiserror::Error)]
pub enum MailgunHookError {
    #[error("failed to decode signature")]
    FailedToDecodeSignature(hex::FromHexError),

    #[error("invalid signature")]
    InvalidSignature(ring::error::Unspecified),

    #[error("server doesn't have a mailgun key configured")]
    NotConfigured,

    #[error("database query failed: {0}")]
    QueryFailed(sqlx::Error),
}

impl IntoResponse for MailgunHookError {
    fn into_response(self) -> Response {
        match &self {
            MailgunHookError::NotConfigured | MailgunHookError::QueryFailed(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "an internal server error occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            _ => {
                let err_msg = serde_json::json!({ "msg": self.to_string() });
                (StatusCode::NOT_ACCEPTABLE, Json(err_msg)).into_response()
            }
        }
    }
}
