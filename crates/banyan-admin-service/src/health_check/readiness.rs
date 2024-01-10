use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::data_source::*;

pub async fn handler(data_src: StateDataSource) -> Response {
    match data_src.is_ready().await {
        Ok(metrics) => {
            // TODO: analyze metrics and handle appropriately
            (StatusCode::OK, Json(metrics)).into_response()
        }
        Err(DataSourceError::DependencyFailure) => {
            let msg = serde_json::json!({"msg": "one or more dependencies aren't available"});
            (StatusCode::SERVICE_UNAVAILABLE, Json(msg)).into_response()
        }
    }
}
