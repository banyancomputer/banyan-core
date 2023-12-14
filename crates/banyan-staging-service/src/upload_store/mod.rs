use std::ops::Deref;
use std::path::PathBuf;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use object_store::{
    aws::AmazonS3,
    local::LocalFileSystem
};

pub use object_store::aws::AmazonS3Builder;

#[derive(Debug, Clone)]
pub enum UploadStoreConnection {
    Local(PathBuf),
    S3(AmazonS3Builder),
}

pub enum UploadStore {
    Local(LocalFileSystem),
    S3(AmazonS3),
}

impl Deref for UploadStore {
    type Target = dyn object_store::ObjectStore;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Local(store) => store,
            Self::S3(store) => store,
        }
    }
}

impl UploadStore {
    pub fn new(connection: &UploadStoreConnection) -> Result<Self, UploadStoreError> {
        match connection {
            UploadStoreConnection::Local(path) => {
                let store = LocalFileSystem::new_with_prefix(path)?;
                Ok(Self::Local(store))
            }
            UploadStoreConnection::S3(builder) => {
                let store = builder.clone().build()?;
                Ok(Self::S3(store))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UploadStoreError {
    #[error("unable to access upload directory")]
    ObjectStore(#[from] object_store::Error),

}

impl IntoResponse for UploadStoreError {
    fn into_response(self) -> Response {
        match self {
            UploadStoreError::ObjectStore(err) => {
                tracing::error!(err = ?err, "configured object store is inaccessible");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}