use axum::routing::{get, post};
use axum::Router;

mod block_retrieval;
mod client_grant;
mod upload;

use crate::app::State;

pub fn router(state: State) -> Router<State> {
    Router::new()
        .route("/blocks/:block_id", get(block_retrieval::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .with_state(state)
}
