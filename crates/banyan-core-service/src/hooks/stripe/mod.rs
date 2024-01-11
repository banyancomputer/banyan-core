use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::extractors::StripeEvent;

pub async fn handler(
    State(_state): State<AppState>,
    _event: StripeEvent,
) -> Response {
    let msg = serde_json::json!({"msg": "ok"});
    (StatusCode::OK, Json(msg)).into_response()
}
