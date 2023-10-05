use axum::Router;

mod error;

// Our routeres
mod device_api_key;
#[cfg(feature = "fake")]
mod fake_account;
mod who_am_i;

pub use error::Error as AuthError;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let r = Router::new();
    #[cfg(feature = "fake")]
    let r = r.nest("/fake_account", fake_account::router(state.clone()));
    r.nest("/device_api_key", device_api_key::router(state.clone()))
        .nest("/who_am_i", who_am_i::router(state.clone()))
        .with_state(state)
}
