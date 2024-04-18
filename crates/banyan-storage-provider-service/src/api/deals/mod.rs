use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, put};
use axum::Router;

use crate::app::AppState;

mod accept_deal;
mod all_deals;
mod reject_deal;

pub use all_deals::DealQuery;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/", get(all_deals::handler))
        .route("/:deal_id/accept", put(accept_deal::handler))
        .route("/:deal_id/reject", put(reject_deal::handler))
        .with_state(state)
}
