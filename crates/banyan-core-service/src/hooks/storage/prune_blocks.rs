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
        let normalized_cid = prune_block.normalized_cid;
        let metadata_id = prune_block.metadata_id;

        let block_id =
            sqlx::query_scalar!(r#"SELECT id FROM blocks WHERE cid = $1;"#, normalized_cid,)
                .fetch_one(&mut *transaction)
                .await
                .map_err(PruneBlocksHookError::SqlxError)?;

        sqlx::query!(
            r#"UPDATE block_locations
            SET pruned_at = CURRENT_TIMESTAMP
            WHERE block_id = $1
                AND metadata_id = $2
                AND storage_host_id = $3;"#,
            block_id,
            metadata_id,
            storage_provider_id,
        )
        .execute(&mut *transaction)
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
    #[error("the task encountered a sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

impl IntoResponse for PruneBlocksHookError {
    fn into_response(self) -> Response {
        {
            tracing::error!("{self}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
