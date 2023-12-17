use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(deal_id): Path<Uuid>,
) -> Response {
    let database = state.database();
    let deal_id = deal_id.to_string();

    let query_result = sqlx::query!(
        r#"UPDATE deals SET state = 'accepted', accepted_by=$1, accepted_at=DATETIME('now') WHERE id = $3 AND state == 'active';"#,
        storage_provider.id,
        deal_id,
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
            tracing::error!("failed to update deal: {err}");
            let err_msg = serde_json::json!({"msg": "a backend service issue occurred"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
