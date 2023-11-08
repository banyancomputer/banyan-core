use std::collections::HashMap;
use std::str::FromStr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cid::multibase::Base;
use cid::Cid;

use crate::app::AppState;
use crate::extractors::UserIdentity;

const NA_LABEL: &str = "NA";

pub type LocationRequest = Vec<String>;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Json(request): Json<LocationRequest>,
) -> Result<Response, BlockLocationError> {
    let database = state.database();

    let user_id = user_identity.id().to_string();
    let mut result_map: HashMap<String, Vec<String>> = HashMap::new();

    for original_cid in request {
        let normalized_cid = Cid::from_str(&original_cid)
            .map_err(BlockLocationError::InvalidCid)?
            .to_string_of_base(Base::Base64Url)
            .map_err(BlockLocationError::InvalidCid)?;

        let block_locations = sqlx::query_scalar!(
            r#"SELECT storage_hosts.url FROM storage_hosts
                   JOIN block_locations ON block_locations.storage_host_id = storage_hosts.id
                   JOIN blocks ON block_locations.block_id = blocks.id
                   JOIN metadata ON metadata.id = block_locations.metadata_id
                   JOIN buckets ON buckets.id = metadata.bucket_id
                   WHERE buckets.user_id = $1
                       AND blocks.cid = $2
                       AND block_locations.expired_at IS NULL
                   ORDER BY RANDOM()
                   LIMIT 5;"#,
            user_id,
            normalized_cid,
        )
        .fetch_all(&database)
        .await
        .map_err(BlockLocationError::LookupFailed)?;

        if block_locations.is_empty() {
            result_map
                .entry(NA_LABEL.to_string())
                .or_default()
                .push(original_cid);
        } else {
            for location in block_locations {
                result_map
                    .entry(location)
                    .or_default()
                    .push(original_cid.clone());
            }
        }
    }

    Ok((StatusCode::OK, Json(result_map)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BlockLocationError {
    #[error("invalid cid provided in request: {0}")]
    InvalidCid(cid::Error),

    #[error("failed to locate storages hosts associated with block: {0}")]
    LookupFailed(sqlx::Error),
}

impl IntoResponse for BlockLocationError {
    fn into_response(self) -> Response {
        match &self {
            BlockLocationError::InvalidCid(_) => {
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
