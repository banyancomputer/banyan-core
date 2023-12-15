use axum::{async_trait, Json};
use axum::http::StatusCode;
use axum::response::{Response, IntoResponse};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app::AppState;
use banyan_object_store::{ObjectStore, ObjectStoreError};

#[async_trait]
impl FromRequestParts<AppState> for ObjectStore {
    type Rejection = ObjectStoreExtractError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let store = Self::new(state.upload_store_connection())?;
        Ok(store)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectStoreExtractError {
    #[error("unable to access object store: {0}")]
    ObjectStore(#[from] ObjectStoreError),
}

impl IntoResponse for ObjectStoreExtractError {
    fn into_response(self) -> Response {
        match self {
            ObjectStoreExtractError::ObjectStore(err) => {
                tracing::error!(err = ?err, "configured object store is inaccessible");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}