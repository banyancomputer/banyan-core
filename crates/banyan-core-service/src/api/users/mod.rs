use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;
use serde::de::StdError;

use crate::app::AppState;

mod read_escrowed_device;
mod read_user;
mod update_user;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn StdError + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route(
            "/current",
            get(read_user::handler).put(update_user::handler),
        )
        .route("/escrowed_device", get(read_escrowed_device::handler))
        .with_state(state)
}
