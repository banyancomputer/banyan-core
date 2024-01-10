use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::app::AppState;
use crate::client::GetAllStorageProvidersRequest;
use crate::extractors::UserIdentity;

pub async fn handler(_: UserIdentity, State(state): State<AppState>) -> Result<Response, ()> {
    let providers = state
        .client()
        .call(GetAllStorageProvidersRequest)
        .await
        .unwrap();

    Ok((StatusCode::OK, Json(providers)).into_response())
}
