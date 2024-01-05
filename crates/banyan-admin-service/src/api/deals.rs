use axum::body::HttpBody;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use http::StatusCode;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
{
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/deals/:deal_id", get(get_deal_handler))
        .route("/deals", get(all_deals_handler))
        .layer(cors_layer)
        .with_state(state)
}

pub async fn all_deals_handler() -> Response {
    (StatusCode::OK, Json(())).into_response()
}

pub async fn get_deal_handler(Path(deal_id): Path<Uuid>) -> Response {
    (StatusCode::OK, Json(())).into_response()
}
