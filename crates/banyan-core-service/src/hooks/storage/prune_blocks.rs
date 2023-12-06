use std::str::FromStr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::extractors::StorageProviderIdentity;
use crate::tasks::PruneBlock;

pub async fn handler(
    State(state): State<AppState>,
    storage_provider_id: StorageProviderIdentity,
    Json(request): Json<Vec<PruneBlock>>,
) -> Result<Response, PruneBlocksHookError> {
    let database = state.database();
    let mut transaction = database
        .begin()
        .await
        .map_err(PruneBlocksHookError::SqlxError)?;
    let storage_provider_id = storage_provider_id.id;

    for prune_block in request {
        // Try and interpret the cid to make sure it's valid
        let _ = cid::Cid::from_str(&prune_block.normalized_cid)
            .map_err(PruneBlocksHookError::InvalidCid)?;

        let metadata_id = prune_block.metadata_id.to_string();

        let block_id = sqlx::query_scalar!(
            r#"SELECT id FROM blocks WHERE cid = $1;"#,
            prune_block.normalized_cid
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(PruneBlocksHookError::SqlxError)?;

        let _pruned_at = sqlx::query!(
            r#"UPDATE block_locations
            SET pruned_at = CURRENT_TIMESTAMP
            WHERE block_id = $1
                AND metadata_id = $2
                AND storage_host_id = $3
            RETURNING pruned_at;"#,
            block_id,
            metadata_id,
            storage_provider_id,
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(PruneBlocksHookError::SqlxError)?;
    }
    transaction
        .commit()
        .await
        .map_err(PruneBlocksHookError::SqlxError)?;
    Ok((StatusCode::OK, ()).into_response())
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksHookError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("invalid cid: {0}")]
    InvalidCid(#[from] cid::Error),
}

impl IntoResponse for PruneBlocksHookError {
    fn into_response(self) -> Response {
        match &self {
            PruneBlocksHookError::InvalidCid(_) => {
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
