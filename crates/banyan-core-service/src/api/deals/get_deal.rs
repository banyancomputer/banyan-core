use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use uuid::Uuid;

use crate::api::models::ApiDeal;
use crate::app::AppState;
use crate::database::models::Deal;
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(deal_id): Path<Uuid>,
) -> Response {
    let database = state.database();
    let deal_id = deal_id.to_string();

    let query_result = sqlx::query_as!(
        Deal,
        r#"SELECT * from deals WHERE id = $1 AND (state='active' OR accepted_by=$2);"#,
        deal_id,
        storage_provider.id
    )
    .fetch_one(&database)
    .await;

    match query_result {
        Ok(b) => (StatusCode::OK, Json(ApiDeal::from(b))).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup specific bucket for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
