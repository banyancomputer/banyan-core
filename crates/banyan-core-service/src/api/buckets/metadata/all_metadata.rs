use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiMetadata;
use crate::app::AppState;
use crate::database::models::PartialMetadataWithSnapshot;
use crate::extractors::UserIdentity;

pub async fn handler(user_identity: UserIdentity, State(state): State<AppState>) -> Response {
    let user_id = user_identity.id().to_string();
    // NOTE: this will not return any metadata in the 'deleted' state
    let query_result = PartialMetadataWithSnapshot::all(&state.database(), user_id).await;

    match query_result {
        Ok(db_meta) => {
            let api_meta: Vec<_> = db_meta.into_iter().map(ApiMetadata::from).collect();
            (StatusCode::OK, Json(api_meta)).into_response()
        }
        Err(err) => {
            tracing::error!("failed to lookup all metadata for account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
