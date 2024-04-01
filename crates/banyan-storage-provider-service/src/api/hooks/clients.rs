use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_object_store::ObjectStoreError;

use crate::api::models::ApiClients;
use crate::app::AppState;
use crate::database::models::Clients;
use crate::extractors::PlatformIdentity;

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(request): Json<ApiClients>,
) -> Result<Response, ClientsCreateError> {
    let db_conn = state.database();
    let client_id = Clients::create_if_missing(
        &db_conn,
        &request.platform_id,
        &request.fingerprint,
        &request.public_key,
    )
    .await?;
    Ok((StatusCode::OK, Json(serde_json::json!({"id": client_id }))).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ClientsCreateError {
    #[error("a database failure occurred: {0}")]
    DbFailure(#[from] sqlx::Error),

    #[error("block retrieval failed: {0}")]
    RetrievalFailed(#[from] ObjectStoreError),

    #[error("request contained invalid CID")]
    InvalidCid,

    #[error("client attempted to access block that wasn't theirs")]
    NotBlockOwner,

    #[error("block not found")]
    UnknownBlock,
}

impl IntoResponse for ClientsCreateError {
    fn into_response(self) -> Response {
        use ClientsCreateError::*;

        match &self {
            DbFailure(err) => {
                tracing::warn!("db failure looking up block: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            RetrievalFailed(err) => {
                tracing::error!("{self}: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            InvalidCid => {
                let err_msg = serde_json::json!({ "msg": format!("invalid cid") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            NotBlockOwner => {
                tracing::warn!("client attempted to access block that wasn't theirs");
                let err_msg = serde_json::json!({ "msg": format!("block not found") });
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            UnknownBlock => {
                let err_msg = serde_json::json!({ "msg": format!("block not found") });
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}
