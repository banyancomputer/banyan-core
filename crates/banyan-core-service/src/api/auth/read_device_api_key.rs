use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
) -> Response {
    let key_id = key_id.to_string();
    let database = state.database();

    let user_id: String = user_identity.id().to_string();
    let query_result =
        sqlx::query_as!(
            DeviceApiKey,
            r#"SELECT id, user_id, fingerprint, pem
               FROM device_api_keys
               WHERE id = $1 AND user_id = $2;"#,
            key_id,
            user_id,
        )
        .fetch_one(&database)
        .await;

    match query_result {
        Ok(dk) => (StatusCode::OK, Json(dk)).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "key not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to remove key from database: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
    }
}

#[derive(sqlx::FromRow, Serialize)]
struct DeviceApiKey {
    id: String,
    user_id: String,
    fingerprint: String,
    pem: String,
}
