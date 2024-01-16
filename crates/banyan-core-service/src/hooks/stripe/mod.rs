use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use stripe::{EventObject, EventType};

use crate::app::AppState;
use crate::extractors::StripeEvent;

pub async fn handler(
    State(_state): State<AppState>,
    StripeEvent(event): StripeEvent,
) -> Response {
    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            if let EventObject::CheckoutSession(session) = event.data.object {
                tracing::info!("recevived checkout session completed webhook with id: {:?}", session.id);
            }
        }
        _ => tracing::warn!("received unknown stripe webhook event: {event:?}"),
    }

    let msg = serde_json::json!({"msg": "ok"});
    (StatusCode::OK, Json(msg)).into_response()
}
