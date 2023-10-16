use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiToken;

pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let account_id = api_token.subject();
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();

    let database = state.database();

    let query_result = sqlx::query!(
            r#"DELETE FROM bucket_keys
                   WHERE id IN (
                       SELECT bk.id FROM bucket_keys AS bk
                           JOIN buckets AS b ON bk.bucket_id = b.id
                           WHERE b.account_id = $1 AND bk.id = $2 AND bk.bucket_id = $3
                   );"#,
            account_id,
            bucket_key_id,
            bucket_id,
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
            tracing::error!("failed to delete bucket key: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
