use std::error::Error;

use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;

use crate::app::AppState;

mod current_escrowed_device;
mod current_user;
mod storage_grant;
mod update_user;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route(
            "/current",
            get(current_user::handler).patch(update_user::handler),
        )
        .route("/escrowed_device", get(current_escrowed_device::handler))
        .route("/storage_grant/:base_url", get(storage_grant::handler))
        .with_state(state)
}
