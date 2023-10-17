use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    let database = state.database();

    // todo: need to delete all the hot data stored at various storage hosts

    let query_result = sqlx::query!(
        r#"DELETE FROM metadata
                   WHERE bucket_id IN
                       (SELECT id FROM buckets WHERE account_id = $1 AND id = $2)
                       AND id = $3;"#,
        api_id.account_id,
        bucket_id,
        metadata_id,
    )
    .execute(&database)
    .await;

    match query_result {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to delete metadata: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
