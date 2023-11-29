use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiBucket;
use crate::app::AppState;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
    Json(bucket): Json<ApiBucket>,
) -> Response {
    let database = state.database();
    let user_id = user_identity.id().to_string();
    let bucket_id = bucket_id.to_string();
    let name = bucket.name;

    let query_result =
        sqlx::query!(
            r#"UPDATE buckets SET name = $1 WHERE id = $2 and user_id = $3;"#,
            name,
            bucket_id,
            user_id
        )
        .execute(&database)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            } else {
                (StatusCode::NO_CONTENT, ()).into_response()
            }
        }
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to update bucket: {err}");
            let err_msg = serde_json::json!({"msg": "a backend service issue occurred"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
