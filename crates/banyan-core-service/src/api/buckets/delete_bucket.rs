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

    // todo: need to delete all the hot data stored at various storage hosts

    let query_result = sqlx::query!(
            r#"DELETE FROM buckets WHERE account_id = $1 AND id = $2;"#,
            account_id,
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
            tracing::error!("failed to delete bucket: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
