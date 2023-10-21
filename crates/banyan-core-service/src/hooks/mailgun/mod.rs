mod event_data;
mod mailgun_event;
mod mailgun_hook_error;
mod mailgun_hook_request;
mod signature;
mod user_variables;

pub(crate) use event_data::EventData;
pub(crate) use mailgun_event::MailgunEvent;
pub(crate) use mailgun_hook_error::MailgunHookError;
pub(crate) use mailgun_hook_request::MailgunHookRequest;
pub(crate) use signature::Signature;
pub(crate) use user_variables::UserVariables;

use axum::Json;
use axum::http::StatusCode;
use axum::extract::State;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;
use crate::database::Database;

pub async fn handler(
    State(state): State<AppState>,
    Json(request): Json<MailgunHookRequest>,
) -> Result<Response, MailgunHookError> {

    let mailgun_webhook_key = state.secrets();

    //request.verify_signature(
    todo!()
}
