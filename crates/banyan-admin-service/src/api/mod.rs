use std::error::Error;
use std::fmt::Display;

use axum::body::HttpBody;
use axum::response::IntoResponse;
use axum::Router;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::app::AppState;
use crate::auth;

mod deals;
mod providers;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
{
    Router::new()
        .nest("/auth/", auth::router(state.clone()))
        .nest("/providers/", providers::router(state.clone()))
        .nest("/deals/", deals::router(state.clone()))
        .layer(CorsLayer::very_permissive())
        .with_state(state)
}
