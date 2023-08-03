use std::fmt::{self, Display, Formatter};

use axum::async_trait;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[derive(Debug)]
pub struct JsonMultipartUpload {
    inner: multer::Multipart<'static>,
}

#[async_trait]
impl<S> FromRequest<S> for JsonMultipartUpload
where
    S: Send + Sync,
{
    type Rejection = JsonMultipartUploadRejection;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        todo!()
    }
}

#[derive(Debug)]
pub struct JsonMultipartUploadRejection(String);

impl Display for JsonMultipartUploadRejection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write(self.0)
    }
}

// dunno if this is needed...
impl std::error::Error for JsonMultipartUploadRejection {}

impl IntoResponse for JsonMultipartUploadRejection {
    fn into_response(self) -> axum::response::Response {
        let err_msg = serde_json::json!({ "error": self.to_string() });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
