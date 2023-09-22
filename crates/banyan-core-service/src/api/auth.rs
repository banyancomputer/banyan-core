use axum::Router;

mod error;

// Our routeres
mod device_api_key;
#[cfg(feature = "fake")]
mod fake_account;
mod who_am_i;

pub use error::Error as AuthError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let r = Router::new();
    #[cfg(feature = "fake")]
    let r = r.nest("/fake_account", fake_account::router(state.clone()));
    r.nest("/device_api_key", device_api_key::router(state.clone()))
        .nest("/who_am_i", who_am_i::router(state.clone()))
        .route("/regwait", get(wait_for_registration))
        .route("/regwait/:handle", get(reg_complete))
        .with_state(state)
}

use std::sync::{Arc, Mutex};

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use crate::app_state::{RegistrationEvent};

pub async fn reg_complete(State(mut state): State<AppState>, Path(channel_name): Path<String>) -> Response {
    tracing::info!("hit reg complete for '{channel_name}'");

    let chan_lock = match state.registration_channels.remove(&channel_name) {
        Some(channel) => channel,
        None => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({"msg": "not found 1"}))).into_response()
        },
    };

    let channel = match chan_lock.lock() {
        Ok(mut chan) => chan.take().unwrap(),
        _ => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"msg": "not found 2"}))).into_response(),
    };

    match channel.send(RegistrationEvent::Rejected) {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"msg": "not found 3"}))).into_response(),
    }
}

pub async fn wait_for_registration(State(mut state): State<AppState>) -> Response {
    let tmp_channel_name = "tmp-channel-name".to_string();

    let (sender, receiver) = tokio::sync::oneshot::channel();
    state.registration_channels.insert(tmp_channel_name, Arc::new(Mutex::new(Some(sender))));

    match tokio::time::timeout(tokio::time::Duration::from_secs(30), receiver).await {
        Ok(chan_result) => {
            match chan_result {
                Ok(RegistrationEvent::Approved(uuid)) => {
                    (StatusCode::OK, Json(serde_json::json!({"account_id": uuid}))).into_response()
                }
                _ => {
                    (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"msg": "device registration rejected"}))).into_response()
                }
            }
        }
        Err(_) => {
            (StatusCode::REQUEST_TIMEOUT, Json(serde_json::json!({"msg": "device registration took too long"}))).into_response()
        }
    }
}
