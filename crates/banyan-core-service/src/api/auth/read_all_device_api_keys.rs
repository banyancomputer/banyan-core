use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::app::AppState;
use crate::extractors::ApiIdentity;

pub async fn handler(api_id: ApiIdentity, State(state): State<AppState>) -> Response {
    let database = state.database();

    let query_result = sqlx::query_as!(
        DeviceApiKey,
        r#"SELECT id, account_id, fingerprint, pem FROM device_api_keys WHERE account_id = $1;"#,
        api_id.account_id,
    )
    .fetch_all(&database)
    .await;

    match query_result {
        Ok(keys) => (StatusCode::OK, Json(keys)).into_response(),
        Err(err) => {
            tracing::error!("failed to query for device keys from the database: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}

#[derive(sqlx::FromRow, Serialize)]
pub struct DeviceApiKey {
    id: String,
    account_id: String,
    fingerprint: String,
    pem: String,
}
