mod event_data;
mod mailgun_event;
mod mailgun_hook_request;
mod signature;
mod user_variables;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
pub(crate) use event_data::EventData;
pub(crate) use mailgun_event::MailgunEvent;
pub(crate) use mailgun_hook_request::MailgunHookRequest;
pub(crate) use signature::Signature;
pub(crate) use user_variables::UserVariables;

use crate::app::AppState;
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

    let database = state.database();
    let mut transaction = database
        .begin()
        .await
        .map_err(MailgunHookError::QueryFailed)?;

    let email_id = match request.message_id() {
        Some(mid) => mid.to_string(),
        None => {
            // This is most likely the test webhook which we need to do no further work with. Just
            // in case log that it occurred to prevent silent failures.
            tracing::warn!("received mailgun test webhook");
            return Ok((StatusCode::OK, ()).into_response());
        }
    };

    let email_state = sqlx::query_as!(
        EmailState,
        "SELECT user_id, state as 'state: EmailMessageState' FROM emails WHERE id = $1;",
        email_id,
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(MailgunHookError::QueryFailed)?
    .ok_or(MailgunHookError::UnknownEmail)?;

    let reported_state = request.event();
    if email_state.state == reported_state {
        return Err(MailgunHookError::DuplicateEvent);
    }

    let stat_sql = format!(
        r#"INSERT INTO email_stats(user_id, {reported_state})
               VALUES ($1, 1)
               ON CONFLICT(user_id) DO UPDATE SET {reported_state} = {reported_state} + 1;"#,
    );

    sqlx::query(&stat_sql)
        .bind(email_state.user_id)
        .execute(&mut *transaction)
        .await
        .map_err(MailgunHookError::QueryFailed)?;

    // If this is old state we don't have anything else to do
    if reported_state < email_state.state {
        transaction
            .commit()
            .await
            .map_err(MailgunHookError::QueryFailed)?;
        return Ok((StatusCode::OK, ()).into_response());
    }

    // Update the email to the new state
    let db_reported_state = reported_state.to_string();

    sqlx::query!(
        "UPDATE emails SET state = $1 WHERE id = $2;",
        db_reported_state,
        email_id,
    )
    .execute(&mut *transaction)
    .await
    .map_err(MailgunHookError::QueryFailed)?;

    transaction
        .commit()
        .await
        .map_err(MailgunHookError::QueryFailed)?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(sqlx::FromRow)]
struct EmailState {
    user_id: String,
    state: EmailMessageState,
}

#[derive(Debug, thiserror::Error)]
pub enum MailgunHookError {
    #[error("received duplicate event")]
    DuplicateEvent,

    #[error("failed to decode signature")]
    FailedToDecodeSignature(hex::FromHexError),

    #[error("invalid signature")]
    InvalidSignature(ring::error::Unspecified),

    #[error("server doesn't have a mailgun key configured")]
    NotConfigured,

    #[error("database query failed: {0}")]
    QueryFailed(sqlx::Error),

    #[error("provided email ID was not located in the database")]
    UnknownEmail,
}

impl IntoResponse for MailgunHookError {
    fn into_response(self) -> Response {
        match &self {
            MailgunHookError::NotConfigured | MailgunHookError::QueryFailed(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "an internal server error occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            MailgunHookError::UnknownEmail => {
                let err_msg = serde_json::json!({ "msg": "email not recognized" });
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                let err_msg = serde_json::json!({ "msg": self.to_string() });
                (StatusCode::NOT_ACCEPTABLE, Json(err_msg)).into_response()
            }
        }
    }
}
