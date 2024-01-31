use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

mod create_device_api_key;
mod create_escrowed_device;
mod delete_device_api_key;
mod read_all_device_api_keys;
mod read_device_api_key;

mod end_regwait;
pub(crate) mod registration_event;
mod start_regwait;

mod who_am_i;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route(
            "/device_api_key",
            get(read_all_device_api_keys::handler).post(create_device_api_key::handler),
        )
        .route(
            "/device_api_key/:key_id",
            get(read_device_api_key::handler).delete(delete_device_api_key::handler),
        )
        .route(
            "/device_api_key/start_regwait/:fingerprint",
            get(start_regwait::handler),
        )
        .route(
            "/device_api_key/end_regwait/:fingerprint",
            get(end_regwait::handler),
        )
        .route(
            "/create_escrowed_device",
            post(create_escrowed_device::handler),
        )
        .route("/who_am_i", get(who_am_i::handler))
        .with_state(state)
}
