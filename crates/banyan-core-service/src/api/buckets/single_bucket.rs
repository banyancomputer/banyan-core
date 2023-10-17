use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiBucket;
use crate::app::AppState;
use crate::database::models::Bucket;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let database = state.database();

    let bucket_id = bucket_id.to_string();

    let query_result = sqlx::query_as!(
        Bucket,
        "SELECT * FROM buckets WHERE account_id = $1 AND id = $2;",
        api_id.account_id,
        bucket_id,
    )
    .fetch_one(&database)
    .await;

    match query_result {
        Ok(b) => (StatusCode::OK, Json(ApiBucket::from(b))).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup specific bucket for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
