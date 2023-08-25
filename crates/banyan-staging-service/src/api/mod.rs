use axum::Router;
use axum::routing::post;

mod client_grant;
mod upload;

use crate::app::State;

pub fn router(state: State) -> Router<State> {
    Router::new()
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .with_state(state)
}
