use axum::routing::post;
use axum::Router;

use crate::app::AppState;

#[cfg(feature = "fake")]
mod create_fake_account;

//mod device_api_key;
//mod who_am_i;

#[cfg(feature = "fake")]
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/fake_account", post(create_fake_account::handler))
        //.nest("/device_api_key", device_api_key::router(state.clone()))
        //.nest("/who_am_i", who_am_i::router(state.clone()))
        .with_state(state)
}

#[cfg(not(feature = "fake"))]
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        //.nest("/device_api_key", device_api_key::router(state.clone()))
        //.nest("/who_am_i", who_am_i::router(state.clone()))
        .with_state(state)
}
