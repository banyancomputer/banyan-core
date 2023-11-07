use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiMetadata;
use crate::app::AppState;
use crate::database::models::PartialMetadataWithSnapshot;
use crate::extractors::{Identity, UserIdentity};

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let user_id = user_id.user_id();
    let query_result = PartialMetadataWithSnapshot::locate_specific(
        &state.database(),
        user_id,
        bucket_id,
        metadata_id,
    )
    .await;

    match query_result {
        Ok(Some(m)) => (StatusCode::OK, Json(ApiMetadata::from(m))).into_response(),
        Ok(None) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup specific metadata for bucket/account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
