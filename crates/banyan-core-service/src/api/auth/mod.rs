use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

mod create_device_api_key;
mod delete_device_api_key;
mod read_all_device_api_keys;
mod read_device_api_key;

mod start_regwait;
mod end_regwait;
pub(crate) mod registration_event;

#[cfg(feature = "fake")]
mod create_fake_account;

mod who_am_i;

#[cfg(feature = "fake")]
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/device_api_key",
            get(read_all_device_api_keys::handler).post(create_device_api_key::handler),
        )
        .route(
            "/device_api_key/:key_id",
            get(read_device_api_key::handler).delete(delete_device_api_key::handler),
        )
        .route("/device_api_key/start_regwait/:fingerprint", get(start_regwait::handler))
        .route("/device_api_key/end_regwait/:fingerprint", get(end_regwait::handler))
        .route("/fake_account", post(create_fake_account::handler))
        .route("/who_am_i", get(who_am_i::handler))
        .with_state(state)
}

#[cfg(not(feature = "fake"))]
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/device_api_key",
            get(read_all_device_api_keys::handler).post(create_device_api_key::handler),
        )
        .route(
            "/device_api_key/:key_id",
            get(read_device_api_key::handler).delete(delete_device_api_key::handler),
        )
        .route("/device_api_key/start_regwait/:fingerprint", get(start_regwait::handler))
        .route("/device_api_key/end_regwait/:fingerprint", get(end_regwait::handler))
        .route("/who_am_i", get(who_am_i::handler))
        .with_state(state)
}
