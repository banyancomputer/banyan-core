use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let bucket_id = bucket_id.to_string();

    let database = state.database();

    tracing::info!(
        "searching for bucket usage on account_id {} with bucket id {}",
        api_id.account_id,
        bucket_id
    );

    let query_count_result: Result<i32, _> = sqlx::query_scalar!(
        r#"SELECT COUNT(id) 
               FROM buckets b 
               WHERE b.account_id = $1 AND b.id = $2;"#,
        api_id.account_id,
        bucket_id,
    )
    .fetch_one(&database)
    .await;

    match query_count_result {
        Ok(query_count) => {
            let query_usage_result: Result<Option<i64>, _> = sqlx::query_scalar!(
                r#"SELECT SUM(m.data_size) as size
                       FROM metadata m
                       JOIN buckets b ON m.bucket_id = b.id
                       WHERE b.account_id = $1 AND b.id = $2;"#,
                api_id.account_id,
                bucket_id,
            )
            .fetch_one(&database)
            .await;

            match query_usage_result {
                Ok(Some(size)) => {
                    let resp = serde_json::json!({ "size": size });
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Ok(None) => {
                    if query_count > 0 {
                        let resp = serde_json::json!({ "size": 0 });
                        (StatusCode::OK, Json(resp)).into_response()
                    } else {
                        let err_msg = serde_json::json!({ "msg": "bucket not found" });
                        (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
                    }
                }
                Err(err) => {
                    tracing::error!("failed to calculate data usage: {err}");
                    let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
                }
            }
        }
        Err(_) => {
            let err_msg = serde_json::json!({ "msg": "bucket not found" });
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
    }
}
