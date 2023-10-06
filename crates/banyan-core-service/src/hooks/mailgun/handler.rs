use axum::extract::{self, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::request::MailgunHookRequest;

use crate::error::CoreError;
use crate::extractors::{DbConn, MailgunSigningKey};
use crate::utils::db::{read_email_state, update_email_state};

pub async fn handle(
    mut db_conn: DbConn,
    mailgun_signing_key: MailgunSigningKey,
    // TODO eng-356: How are these errors handled?
    Json(hook_request): extract::Json<MailgunHookRequest>,
) -> Response {
    match hook_request.verify_signature(&mailgun_signing_key.0) {
        Ok(_) => (),
        Err(err) => {
            return CoreError::generic_error(
                StatusCode::NOT_ACCEPTABLE,
                "invalid signature",
                Some(&format!("failed to verify signature: {err}")),
            )
            .into_response()
        }
    }

    let message_id = hook_request.message_id();
    let next_state = hook_request.event();

    let maybe_current_state = read_email_state(message_id, &mut db_conn).await;
    let current_state = match maybe_current_state {
        Ok(state) => state,
        Err(err) => {
            return CoreError::generic_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backend service issue",
                Some(&format!("failed to read message state: {err}")),
            )
            .into_response()
        }
    };

    // No need to do anything if the current state is already greater than or equal to the next state
    if next_state < current_state {
        return (StatusCode::OK).into_response();
    }
    // If we get this then something went wrong on Mailgun's end
    if next_state == current_state {
        return CoreError::generic_error(
            StatusCode::NOT_ACCEPTABLE,
            "received duplicate event",
            Some("received duplicate event"),
        )
        .into_response();
    }

    // Otherwise, this is a valid state transition
    let maybe_updated_email = update_email_state(message_id, next_state, &mut db_conn).await;

    match maybe_updated_email {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(err) => CoreError::generic_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "backend service issue",
            Some(&format!("failed to update message state: {err}")),
        )
        .into_response(),
    }
}
