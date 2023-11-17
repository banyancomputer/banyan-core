use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use axum::async_trait;
use object_store::local::LocalFileSystem;

use crate::app::AppState;
use crate::upload_store::{UploadStore, UploadStoreError};

// TODO: it would be preferable to extract the upload store from the request state,
// but since LocalFileSystem doesn't implement Clone, we can't do that just yet.
// As implemented right now, this whole thing is a bit redundant.
#[async_trait]
impl FromRequestParts<AppState> for UploadStore {
    type Rejection = UploadStoreError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Self::new(&state.upload_directory())
    }
}

