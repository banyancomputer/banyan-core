use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::PartialMetadataWithSnapshot;
use crate::extractors::ApiToken;
use crate::api::models::ApiMetadata;

pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let database = state.database();

    let query_result = PartialMetadataWithSnapshot::locate_current(
        &state.database(),
        api_token.subject(),
        bucket_id,
    ).await;

    match query_result {
        Ok(Some(m)) => (StatusCode::OK, Json(ApiMetadata::from(m))).into_response(),
        Ok(None) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            return (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup current metadata for bucket/account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
