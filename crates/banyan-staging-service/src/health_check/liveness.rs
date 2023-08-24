use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// This is a very simple handler that always returns with a valid response. It's intended to be
/// used by external healthchecks to see whether the service is "alive". Failing this check for any
/// reason generally leads to immediate termination of the service.
///
/// If you're looking for how to report a service issue, please refer to
/// [`crate::health_check::readiness::handler`].
pub async fn handler() -> Response {
    let msg = serde_json::json!({"status": "ok"});
    (StatusCode::OK, Json(msg)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_direct() {
        let response = handler().await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"{\"status\":\"ok\"}");
    }
}
