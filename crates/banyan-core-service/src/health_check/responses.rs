use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use banyan_task::SqliteTaskStoreMetrics;

use crate::health_check::HealthCheckError;
use crate::utils::collect_error_messages;

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum Response {
    Error { errors: Vec<String> },
    Pending { message: String },
    Ready,
    TaskStoreMetrics(SqliteTaskStoreMetrics),
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
            _ => StatusCode::OK,
        };

        (status_code, Json(self)).into_response()
    }
}
