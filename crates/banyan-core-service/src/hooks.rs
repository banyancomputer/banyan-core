use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Serialize;

mod mailgun;

use crate::app_state::AppState;
use crate::utils::collect_error_messages;

pub fn router(state: AppState) -> Router<AppState> {
    // TODO: Find the right cors config for this
    let cors_layer = tower_http::cors::CorsLayer::very_permissive();

    Router::new()
        .nest("/mailgun", mailgun::router(state.clone()))
        .with_state(state)
        .layer(cors_layer)
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub errors: Vec<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

macro_rules! from_error_impl {
    ($t: ty) => {
        impl From<$t> for ErrorResponse {
            fn from(err: $t) -> Self {
                Self {
                    errors: collect_error_messages(err),
                }
            }
        }
    };
}

from_error_impl!(mailgun::MailgunHookError);
