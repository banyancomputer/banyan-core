use std::error::Error;

use axum::body::HttpBody;
use axum::response::{IntoResponse, Response};
use axum::Router;
use tower_http::cors::CorsLayer;

mod admin;
mod auth;
mod blocks;
mod buckets;
mod deals;
mod invoices;
pub mod models;
mod share;
mod subscriptions;
mod users;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    // TODO: Ideally this would have a wrapper method to allow per route method configuration or
    // even better something that inspected the route matches and applied the correct method config
    // for that path...
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .nest("/admin", admin::router(state.clone()))
        .nest("/auth", auth::router(state.clone()))
        .nest("/blocks", blocks::router(state.clone()))
        .nest("/buckets", buckets::router(state.clone()))
        .nest("/deals", deals::router(state.clone()))
        .nest("/invoices", invoices::router(state.clone()))
        .nest("/share", share::router(state.clone()))
        .nest("/subscriptions", subscriptions::router(state.clone()))
        .nest("/users", users::router(state.clone()))
        .layer(cors_layer)
        .with_state(state)
        .fallback(api_not_found_handler)
}

pub async fn api_not_found_handler() -> Response {
    let err_msg = serde_json::json!({"msg": "not found"});
    (http::StatusCode::NOT_FOUND, axum::Json(err_msg)).into_response()
}
