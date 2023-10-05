use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::error::MailgunHookError;
use super::request::MailgunHookRequest;

use crate::extractors::{DbConn, MailgunSigningKey};
use crate::utils::db::{read_email_state, update_email_state};

struct GenericError {
    code: StatusCode,
    msg: String,
}

impl GenericError {
    pub fn new(code: StatusCode, msg: impl ToString) -> Self {
        Self {
            code,
            msg: msg.to_string(),
        }
    }
}

impl IntoResponse for GenericError {
    fn into_response(self) -> Response {
        (self.code, Json(serde_json::json!({"msg": self.msg}))).into_response()
    }
}

pub async fn handle(
    mut db_conn: DbConn,
    mailgun_signing_key: MailgunSigningKey,
    Json(hook_request): extract::Json<MailgunHookRequest>,
) -> Response {
    // Verify the signature
    match hook_request.verify(&mailgun_signing_key.0) {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("failed to verify signature: {err}");
            return MailgunHookError::invalid_signature().into_response();
        }
    }

    let message_id = hook_request.message_id();
    let event = hook_request.event();

    let maybe_current_state = read_email_state(message_id, &mut db_conn).await;
    let current_state = match maybe_current_state {
        Ok(state) => state,
        Err(err) => {
            tracing::error!("failed to read message state: {err}");
            return GenericError::new(StatusCode::INTERNAL_SERVER_ERROR, "backend service issue")
                .into_response();
        }
    };

    let next_state = event.into();

    // No need to do anything if the current state is already greater than or equal to the next state
    if next_state < current_state {
        return (StatusCode::OK).into_response();
    }
    // If we get this then something went wrong on Mailgun's end
    if next_state == current_state {
        return MailgunHookError::out_of_order_event().into_response();
    }

    let maybe_updated_email = update_email_state(message_id, next_state, &mut db_conn).await;

    match maybe_updated_email {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(err) => {
            tracing::error!("failed to update message state: {err}");
            GenericError::new(StatusCode::INTERNAL_SERVER_ERROR, "backend service issue")
                .into_response()
        }
    }
}
