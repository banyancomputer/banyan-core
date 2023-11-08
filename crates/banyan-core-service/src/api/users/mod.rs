use axum::routing::get;
use axum::Router;

use crate::app::AppState;

mod read_user;
mod update_user;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/current",
            get(read_user::handler).put(update_user::handler)
        )
        .with_state(state)
}
