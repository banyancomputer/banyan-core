use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json};
use object_store::local::LocalFileSystem;

use crate::app::State;

#[derive(Debug)]
pub struct UploadStore(LocalFileSystem);

impl Deref for UploadStore {
    type Target = LocalFileSystem;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl FromRequestParts<State> for UploadStore {
    type Rejection = UploadStoreError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let store = match LocalFileSystem::new_with_prefix(state.upload_directory()) {
            Ok(s) => s,
            Err(err) => return Err(UploadStoreError::DirectoryUnavailable(err)),
        };

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
        use UploadStoreError::*;

        match self {
            DirectoryUnavailable(err) => {
                tracing::error!(err = ?err, "upload directory was unavailable for request");
                let err_msg = serde_json::json!({ "msg": "data uploads currently unavailable due to service error" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
