use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::{Identity, UserIdentity};

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let bucket_id = bucket_id.to_string();

    let database = state.database();

    let user_id = user_id.user_id();
    let query_result: Result<i32, _> = sqlx::query_scalar!(
        r#"SELECT COALESCE(SUM(m.data_size), 0) as size
               FROM metadata m
               JOIN buckets b ON m.bucket_id = b.id
               WHERE b.user_id = $1 AND b.id = $2;"#,
        user_id,
        bucket_id,
    )
    .fetch_one(&database)
    .await;

    match query_result {
        Ok(size) => {
            let resp = serde_json::json!({ "size": size });
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(_) => {
            let err_msg = serde_json::json!({ "msg": "bucket not found" });
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
    }
}
