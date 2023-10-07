use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::Metadata;
use crate::extractors::ApiToken;
use crate::api::common::ApiMetadata;

pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> Response {
    let database = state.database();

    let account_id = api_token.subject();
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    let query_result = sqlx::query_as!(
        Metadata,
            r#"SELECT m.* FROM metadata m
                  JOIN buckets b ON m.bucket_id = b.id
                  WHERE m.id = $1 AND b.account_id = $2 AND b.id = $3;"#,
        metadata_id,
        account_id,
        bucket_id,
    )
    .fetch_one(&database)
    .await;

    let metadata = match query_result {
        Ok(b) => b,
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            return (StatusCode::NOT_FOUND, Json(err_msg)).into_response();
        }
        Err(err) => {
            tracing::error!("failed to lookup specific metadata for bucket/account: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response();
        }
    };

    let query_result = sqlx::query_scalar!(
            "SELECT id FROM snapshots WHERE metadata_id = $1;",
            metadata.id,
        )
        .fetch_one(&database)
        .await;

    let snapshot_id: Option<String> = match query_result {
        Ok(sid) => Some(sid),
        Err(sqlx::Error::RowNotFound) => None,
        Err(err) => {
            tracing::error!("failed to lookup snapshot for metadata: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response();
        }
    };

    let api_metadata = ApiMetadata {
        id: metadata.id,

        root_cid: metadata.root_cid,
        metadata_cid: metadata.metadata_cid,
        data_size: metadata.data_size.unwrap_or(0),

        state: metadata.state,

        created_at: metadata.created_at.unix_timestamp(),
        updated_at: metadata.updated_at.unix_timestamp(),

        snapshot_id,
    };

    (StatusCode::OK, Json(api_metadata)).into_response()
}
