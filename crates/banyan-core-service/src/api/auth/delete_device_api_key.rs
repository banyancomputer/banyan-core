use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Response {
    let key_id = key_id.to_string();
    let database = state.database();

    let query_result = sqlx::query!(
        r#"DELETE FROM device_api_keys
            WHERE id = $1 AND user_id = $2;"#,
        key_id,
        api_id.user_id,
    )
    .execute(&database)
    .await;

    match query_result {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "key not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to remove key from database: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
