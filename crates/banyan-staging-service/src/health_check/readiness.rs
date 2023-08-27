use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::data_source::*;

pub async fn handler(data_src: StateDataSource) -> Response {
    match data_src.is_ready().await {
        Ok(_) => {
            let msg = serde_json::json!({"status": "ok"});
            (StatusCode::OK, Json(msg)).into_response()
        },
        Err(DataSourceError::DependencyFailure) => {
            let msg = serde_json::json!({"status": "failure", "message": "one or more dependencies aren't available"});
            (StatusCode::SERVICE_UNAVAILABLE, Json(msg)).into_response()
        },
        Err(DataSourceError::ShuttingDown) => {
            let msg = serde_json::json!({"status": "failure", "message": "service is shutting down"});
            (StatusCode::SERVICE_UNAVAILABLE, Json(msg)).into_response()
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    use crate::health_check::data_source::tests::*;

    #[tokio::test]
    async fn test_handler_direct() {
        let response = handler(StateDataSource::new(Arc::new(MockReadiness::Ready))).await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = handler(StateDataSource::new(Arc::new(MockReadiness::DependencyFailure))).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let response = handler(StateDataSource::new(Arc::new(MockReadiness::ShuttingDown))).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
