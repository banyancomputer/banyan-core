use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::extractors::ApiIdentity;
use crate::utils::is_valid_cid;

const NA_LABEL: &str = "NA";

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Json(cid_list): Json<Vec<String>>,
) -> Result<Response, BlockLocationError> {
    let database = state.database();

    let user_id = api_id.user_id().to_string();
    let mut result_map: HashMap<String, Vec<String>> = HashMap::new();

    for cid in cid_list.iter() {
        // todo(sstelfox): The CID crate only supports parsing data from 512 bit hashes which we
        // don't use exclusively.
        if !is_valid_cid(cid) {
            return Err(BlockLocationError::InvalidCid);
        }

        let block_locations = sqlx::query_scalar!(
            r#"SELECT storage_hosts.url FROM storage_hosts
                   JOIN block_locations ON block_locations.storage_host_id = storage_hosts.id
                   JOIN blocks ON block_locations.block_id = blocks.id
                   JOIN metadata ON metadata.id = block_locations.metadata_id
                   JOIN buckets ON buckets.id = metadata.bucket_id
                   WHERE buckets.user_id = $1
                       AND blocks.cid = $2
                       AND block_locations.expired_at IS NULL
                       AND block_locations.stored_at IS NOT NULL
                   ORDER BY RANDOM()
                   LIMIT 5;"#,
            user_id,
            cid,
        )
        .fetch_all(&database)
        .await
        .map_err(BlockLocationError::LookupFailed)?;

        if block_locations.is_empty() {
            result_map
                .entry(NA_LABEL.to_string())
                .or_default()
                .push(cid.clone());
        } else {
            for location in block_locations {
                result_map.entry(location).or_default().push(cid.clone());
            }
        }
    }

    Ok((StatusCode::OK, Json(result_map)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BlockLocationError {
    #[error("invalid CID provided in request")]
    InvalidCid,

    #[error("failed to locate storages hosts associated with block: {0}")]
    LookupFailed(sqlx::Error),
}

impl IntoResponse for BlockLocationError {
    fn into_response(self) -> Response {
        match &self {
            BlockLocationError::InvalidCid => {
                let err_msg = serde_json::json!({"msg": "invalid CID provided in the list"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
