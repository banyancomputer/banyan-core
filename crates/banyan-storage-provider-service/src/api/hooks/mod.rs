mod blocks;
mod clients;
mod get_clients;
mod get_uploads;
mod uploads;

use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    Router::new()
        .route("/clients", post(clients::handler))
        .route("/uploads", post(uploads::handler))
        .route("/metadata/:metadata_id/uploads", get(get_uploads::handler))
        .route("/metadata/:metadata_id/clients", get(get_clients::handler))
        .route("/blocks", post(blocks::handler))
        .with_state(state)
}
