use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::api::models::ApiUserKey;
use crate::app::AppState;
use crate::database::models::UserKey;
use crate::extractors::UserIdentity;

pub async fn handler(user_identity: UserIdentity, State(state): State<AppState>) -> Response {
    let database = state.database();

    let user_id = user_identity.id().to_string();
    let query_result = sqlx::query_as!(
        UserKey,
        r#"
            SELECT *
            FROM user_keys
            WHERE user_id = $1;
        "#,
        user_id,
    )
    .fetch_all(&database)
    .await;

    match query_result {
        Ok(keys) => (
            StatusCode::OK,
            Json(keys.into_iter().map(ApiUserKey::from).collect::<Vec<_>>()),
        )
            .into_response(),
        Err(err) => {
            tracing::error!("failed to query for device keys from the database: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
