use std::ops::Deref;
use std::path::PathBuf;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use object_store::local::LocalFileSystem;

pub struct UploadStore(LocalFileSystem);

impl Deref for UploadStore {
    type Target = LocalFileSystem;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UploadStore {
    pub fn new(upload_path: &PathBuf) -> Result<Self, UploadStoreError> {
        let store = LocalFileSystem::new_with_prefix(upload_path)
            .map_err(UploadStoreError::DirectoryUnavailable)?;
        Ok(Self(store))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UploadStoreError {
    #[error("unable to access upload directory")]
    DirectoryUnavailable(object_store::Error),
}

impl IntoResponse for UploadStoreError {
    fn into_response(self) -> Response {
        match self {
            UploadStoreError::DirectoryUnavailable(err) => {
                tracing::error!(err = ?err, "upload directory was unavailable for request");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}

pub type UploadStoreResult<T> = Result<T, UploadStoreError>;