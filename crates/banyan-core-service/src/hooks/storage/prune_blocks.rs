use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cid::Cid;

use crate::app::AppState;
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
            Err(err) => tracing::warn!(storage_provider_id, "storage provider reported invalid CID: {err}"),
        }
    }

    // Retrieve the database identifiers for any blocks we know about matching the provided CIDs
    let mut block_id_query = sqlx::QueryBuilder::new("SELECT id FROM blocks WHERE cid IN (");

    let mut cid_list_iterator = normalized_cid_list.into_iter().peekable();
    while let Some(cid) = cid_list_iterator.next() {
        block_id_query.push_bind(cid);

        if cid_list_iterator.peek().is_some() {
            block_id_query.push(", ");
        }
    }

    block_id_query.push(");");
    let block_ids: Vec<String> = block_id_query.build_query_scalar().persistent(false).fetch_all(&mut *conn).await?;

    // Mark only the associations from that particular storage provider from our collected
    // database blocks IDs as pruned
    let mut prune_builder = sqlx::QueryBuilder::new(
        r#"UPDATE block_locations SET pruned_at = DATETIME('now')
               WHERE storage_provider_id = "#,
    );
    prune_builder.push_bind(&storage_provider_id);
    prune_builder.push(" AND block_id IN (");

    let mut block_id_iterator = block_ids.iter().peekable();
    while let Some(bid) = block_id_iterator.next() {
        prune_builder.push_bind(bid);

        if block_id_iterator.peek().is_some() {
            prune_builder.push(", ");
        }
    }

    prune_builder.push(");");
    let prune_result = prune_builder.build().execute(&mut *conn).await?;

    conn.commit().await?;

    tracing::debug!(pruned_blocks = prune_result.rows_affected(), "blocks pruned from storage provider");

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
