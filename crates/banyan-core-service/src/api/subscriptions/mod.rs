mod all_subscriptions;
mod manage_subscription;
mod purchase_subscription;
mod single_subscription;

mod session_returns;

use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    Router::new()
        .route("/:subscription_id", get(single_subscription::handler))
        .route(
            "/:subscription_id/subscribe",
            post(purchase_subscription::handler).get(purchase_subscription::handler),
        )
        .route("/cancel", get(session_returns::cancel_redirect))
        .route("/manage", get(manage_subscription::handler))
        .route("/success/:checkout_session_id", get(session_returns::success_redirect))
        .route("/", get(all_subscriptions::handler))
        .with_state(state)
}
