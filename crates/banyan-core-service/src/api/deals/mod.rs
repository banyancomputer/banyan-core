mod accept_deal;
mod all_deals;
mod cancel_deal;
mod get_deal;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
{
    Router::new()
        .route("/", get(all_deals::handler))
        .route("/:deal_id", get(get_deal::handler))
        .route("/:deal_id/accept", post(accept_deal::handler))
        .route("/:deal_id/cancel", post(cancel_deal::handler))
        .with_state(state)
}
