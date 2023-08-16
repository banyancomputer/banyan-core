use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::app_state::AppState;

#[derive(Debug)]
pub struct StorageHost(String);

impl StorageHost {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }
}

#[async_trait]
impl FromRequestParts<AppState> for StorageHost {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let storage_host = app_state.storage_host_url().clone();
        Ok(StorageHost(storage_host.to_string()))
    }
}
