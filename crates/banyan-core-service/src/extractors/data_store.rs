use std::ops::Deref;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use banyan_object_store::{ObjectStore, ObjectStoreConnection};

use crate::app::AppState;
/*
#[derive(Debug)]
pub struct DataStore(LocalFileSystem);

impl Deref for DataStore {
    type Target = LocalFileSystem;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

*/

#[async_trait]
impl FromRequestParts<AppState> for ObjectStore {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let store = match ObjectStore::new(&ObjectStoreConnection::Local(state.upload_directory()))
        {
            Ok(s) => s,
            Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
        };

        Ok(store)
    }
}
