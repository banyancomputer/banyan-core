use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;
use crate::database::models::PartialMetadataWithSnapshot;
use crate::extractors::ApiIdentity;
use crate::api::models::ApiMetadata;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
) -> Response {
    let query_result = PartialMetadataWithSnapshot::all(
        &state.database(),
        api_id.account_id,
    ).await;

    match query_result {
        Ok(db_meta) => {
            let api_meta: Vec<_> = db_meta.into_iter().map(|m| ApiMetadata::from(m)).collect();
            (StatusCode::OK, Json(api_meta)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup all metadata for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
