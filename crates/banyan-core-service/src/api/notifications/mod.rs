use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{delete, get};
use axum::Router;

mod all_notifications;
mod delete_notification;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    Router::new()
        .route("/", get(all_notifications::handler))
        .route("/:notification_id", delete(delete_notification::handler))
        .with_state(state)
}
