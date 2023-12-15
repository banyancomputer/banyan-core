use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app::AppState;
use banyan_object_store::{ObjectStore, ObjectStoreError};

// TODO: it would be preferable to extract the upload store from the request state,
// but since LocalFileSystem doesn't implement Clone, we can't do that just yet.
// As implemented right now, this whole thing is a bit redundant.
#[async_trait]
impl FromRequestParts<AppState> for ObjectStore {
    type Rejection = ObjectStoreError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Self::new(state.upload_store_connection())
    }
}