use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

mod create_escrowed_user_key;
mod create_user_key;
mod rename_user_key;

mod read_all_user_keys;
mod read_user_key;
mod user_key_access;

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
        .route("/user_key_access", get(user_key_access::handler))
        .route(
            "/user_key",
            get(read_all_user_keys::handler).post(create_user_key::handler),
        )
        .route(
            "/user_key/:key_id",
            get(read_user_key::handler).post(rename_user_key::handler),
        )
        .route(
            "/user_key/start_regwait/:fingerprint",
            get(start_regwait::handler),
        )
        .route(
            "/user_key/end_regwait/:fingerprint",
            get(end_regwait::handler),
        )
        .route(
            "/create_escrowed_user_key",
            post(create_escrowed_user_key::handler),
        )
        .route("/who_am_i", get(who_am_i::handler))
        .with_state(state)
}
