use crate::app::AppState;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use jwt_simple::algorithms::ECDSAP384PublicKeyLike;
use serde_json::json;

pub async fn handler(State(state): State<AppState>) -> Response {
    // Grab the verifcation key from the state secrets
    let verification_key = state.secrets().service_key().verifier();

    // If we can succesfully represent the app state public key as PEM
    if let Ok(pem) = (*verification_key).public_key().to_pem() {
        (StatusCode::OK, Json(json!({ "pem": pem }))).into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"msg": "internal server error"})),
        )
            .into_response()
    }
}
