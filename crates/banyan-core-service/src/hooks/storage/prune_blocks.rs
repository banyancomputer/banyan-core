use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cid::Cid;

use crate::app::AppState;
use crate::database::BIND_LIMIT;
use crate::extractors::StorageProviderIdentity;
use crate::utils::NORMALIZED_CID_BASE;

pub async fn handler(
    State(state): State<AppState>,
    storage_provider_id: StorageProviderIdentity,
    Json(cid_list): Json<Vec<Cid>>,
) -> Result<Response, PruneBlocksHookError> {
    let database = state.database();
    let mut conn = database.begin().await?;

    let storage_provider_id = storage_provider_id.id;

    // Normalize and clean up the CID list into only valid ones
    let mut normalized_cid_list = Vec::new();
    for cid in cid_list.into_iter() {
        match cid.to_string_of_base(NORMALIZED_CID_BASE) {
            Ok(n_cid) => normalized_cid_list.push(n_cid),
            Err(err) => tracing::warn!(
                storage_provider_id,
                "storage provider reported invalid CID: {err}"
            ),
        }
    }

    // Retrieve the database identifiers for any blocks we know about matching the provided CIDs
    let mut block_ids = Vec::new();
    for chunk in normalized_cid_list.chunks(BIND_LIMIT) {
        let mut block_id_query = sqlx::QueryBuilder::new("SELECT id FROM blocks WHERE cid IN (");

        let mut separated_values = block_id_query.separated(", ");
        for cid in chunk {
            separated_values.push_bind(cid);
        }

        block_id_query.push(");");

        let queried_ids: Vec<String> = block_id_query
            .build_query_scalar()
            .persistent(false)
            .fetch_all(&mut *conn)
            .await?;

        block_ids.extend(queried_ids);
    }

    // Mark only the associations from that particular storage provider from our collected
    // database blocks IDs as pruned
    for chunk in block_ids.chunks(BIND_LIMIT) {
        let mut prune_builder = sqlx::QueryBuilder::new(
            r#"UPDATE block_locations SET pruned_at = CURRENT_TIMESTAMP
                WHERE storage_provider_id = "#,
        );
        prune_builder.push_bind(&storage_provider_id);
        prune_builder.push(" AND block_id IN (");

        let mut separated_values = prune_builder.separated(", ");
        for id in chunk {
            separated_values.push_bind(id);
        }

        prune_builder.push(");");
        let prune_result = prune_builder
            .build()
            .persistent(false)
            .execute(&mut *conn)
            .await?;

        tracing::debug!(
            pruned_blocks = prune_result.rows_affected(),
            "blocks pruned from storage provider"
        );
    }

    conn.commit().await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksHookError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

impl IntoResponse for PruneBlocksHookError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
