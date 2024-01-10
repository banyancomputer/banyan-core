use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;
use crate::client::CreateStorageProvidersRequest;
use crate::extractors::UserIdentity;

pub async fn handler(
    _: UserIdentity,
    State(state): State<AppState>,
    Json(request): Json<CreateStorageProvidersRequest>,
) -> Result<Response, ()> {
    let result = state.client().call(request).await.unwrap();

    Ok((StatusCode::OK, Json(result)).into_response())
}
