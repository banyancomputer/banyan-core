use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

mod create_api_key;
mod create_escrowed_device;
mod delete_api_key;
mod read_all_api_keys;
mod read_api_key;

mod end_regwait;
pub(crate) mod registration_event;
mod start_regwait;

mod provider_grant;
mod who_am_i;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route(
            "/provider_grant/:storage_host_id",
            get(provider_grant::handler),
        )
        .route(
            "/device_api_key",
            get(read_all_api_keys::handler).post(create_api_key::handler),
        )
        .route(
            "/device_api_key/:key_id",
            get(read_api_key::handler).delete(delete_api_key::handler),
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
