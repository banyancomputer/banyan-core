use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::database::models::Clients;
use crate::extractors::PlatformIdentity;

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<String>,
) -> Result<Response, GetClientError> {
    let db = state.database();
    let client = match Clients::find_bu_metadata_id(&db, &metadata_id).await? {
        Some(client) => client,
        None => return Err(GetClientError::ClientNotFound),
    };
    let msg = serde_json::json!({
        "platform_id": client.platform_id,
        "fingerprint": client.fingerprint,
        "public_key": client.public_key,
    });
    Ok((StatusCode::OK, Json(msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum GetClientError {
    #[error("failed to lookup client: {0}")]
    LookupFailed(#[from] sqlx::Error),
    #[error("client not found")]
    ClientNotFound,
}

impl IntoResponse for GetClientError {
    fn into_response(self) -> Response {
        match self {
            GetClientError::ClientNotFound => {
                let err_msg = serde_json::json!({"msg": "client not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                let err_msg = serde_json::json!({"msg": "a backend service issue occurred"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
