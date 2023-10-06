use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiToken;

pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let account_id = api_token.subject();
    let bucket_id = bucket_id.to_string();

    let database = state.database();

    let query_result: Result<Option<i64>, _> = sqlx::query_scalar!(
        r#"SELECT SUM(m.data_size) as size
               FROM metadata m
               JOIN buckets b ON m.bucket_id = b.id
               WHERE b.account_id = $1 AND b.id = $2;"#,
        account_id,
        bucket_id,
    )
    .fetch_one(&database)
    .await;

    match query_result {
        Ok(Some(size)) => {
            let resp = serde_json::json!({ "size": size });
            (StatusCode::OK, Json(resp)).into_response()
        }
        Ok(None) => {
            let err_msg = serde_json::json!({ "msg": "bucket not found" });
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to calculate data usage: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
