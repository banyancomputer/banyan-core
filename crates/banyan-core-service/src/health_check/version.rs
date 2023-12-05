use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::app::Version;

pub async fn handler() -> Response {
    (StatusCode::OK, Json(Version::new())).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_direct() {
        let response = handler().await;
        assert_eq!(response.status(), StatusCode::OK);
        // TODO: this is a bit fragile, but it's the best we can do for now
    }
}
