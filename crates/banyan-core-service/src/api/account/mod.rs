use axum::routing::get;
use axum::Router;

use crate::app_state::AppState;

mod account_info;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/info", get(account_info::handler))
        .with_state(state)
}
