use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::BucketKey;
use crate::extractors::ApiToken;
use crate::api::models::ApiBucketKey;

pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let database = state.database();

    let account_id = api_token.subject();
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();

    let query_result = sqlx::query_as!(
        BucketKey,
        r#"SELECT * FROM bucket_keys WHERE account_id = $1 AND id = $2;"#,
        account_id,
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
