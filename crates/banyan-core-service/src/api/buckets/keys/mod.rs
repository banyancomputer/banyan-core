use axum::routing::{get, post};
use axum::Router;

mod all_bucket_keys;
mod create_bucket_key;
mod delete_bucket_key;
mod reject_bucket_key;
mod single_bucket_key;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(all_bucket_keys::handler).post(create_bucket_key::handler),
        )
        .route(
            "/:bucket_key_id",
            get(single_bucket_key::handler).delete(delete_bucket_key::handler),
        )
        .route("/:bucket_key_id/reject", post(reject_bucket_key::handler))
        .with_state(state)
}
