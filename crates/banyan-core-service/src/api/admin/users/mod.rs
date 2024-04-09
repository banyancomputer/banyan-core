use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{delete, get};
use axum::Router;

mod all_users;
mod reset_user;
use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/", get(all_users::handler))
        .route("/:user_id", delete(reset_user::handler))
        .with_state(state)
}
