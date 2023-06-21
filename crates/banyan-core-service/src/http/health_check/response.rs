use axum::Json;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::error::collect_error_messages;
use crate::http::health_check::HealthCheckError;

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum Response {
    Error { errors: Vec<String> },
    Pending { message: String },
    Ready,
}

impl From<HealthCheckError> for Response {
    fn from(value: HealthCheckError) -> Self {
        if value.is_temporary() {
            Response::Pending {
                message: value.to_string(),
            }
        } else {
            Response::Error {
                errors: collect_error_messages(value),
            }
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        use Response::*;

        let status_code = match self {
            Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Pending { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Ready => StatusCode::OK,
        };

        (status_code, Json(self)).into_response()
    }
}
