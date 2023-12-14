use std::collections::BTreeSet;

use axum::debug_handler;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cid::multibase::Base;
use cid::Cid;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::UserIdentity;

#[debug_handler]
pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<BTreeSet<Cid>>,
) -> Result<Response, CreateSnapshotError> {
    let database = state.database();
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    let user_id = user_identity.id().to_string();
    let metadata_id = sqlx::query_scalar!(
        r#"SELECT m.id FROM metadata AS m
               JOIN buckets AS b ON m.bucket_id = b.id
               LEFT JOIN snapshots AS s ON s.metadata_id = m.id
               WHERE b.user_id = $1
                   AND b.id = $2
                   AND m.id = $3
                   AND s.id IS NULL;"#,
        user_id,
        bucket_id,
        metadata_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(CreateSnapshotError::MetadataUnavailable)?
    .ok_or(CreateSnapshotError::NotFound)?;

    tracing::info!(
        "owned_metadata_id: {}, metadata_id: {}",
        metadata_id,
        metadata_id
    );

    let snapshot_id = sqlx::query_scalar!(
        r#"INSERT INTO snapshots (metadata_id, state)
               VALUES ($1, 'pending')
               RETURNING id;"#,
        metadata_id,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateSnapshotError::SaveFailed)?;

    // Normalize all the CIDs
    let normalized_cids = request
        .into_iter()
        .map(|cid| {
            cid.to_string_of_base(Base::Base64Url)
                .map_err(CreateSnapshotError::InvalidInternalCid)
        })
        .collect::<Result<Vec<_>, _>>()?;

    tracing::info!(
        "developing associations with these cids for snapshot: {:?}",
        normalized_cids
    );

    // Create query builder that can serve as the basis for every chunk
    let mut builder = sqlx::QueryBuilder::new(format!(
        "INSERT INTO snapshot_block_locations 
            SELECT s.id as snapshot_id, bl.block_id 
            FROM blocks AS b 
            JOIN block_locations AS bl ON b.id = bl.block_id 
            JOIN metadata AS m ON bl.metadata_id = m.id 
            JOIN snapshots AS s 
            WHERE m.id = \"{metadata_id}\"
            AND s.id = \"{snapshot_id}\"
            AND b.cid IN ("
    ));

    // For every chunk of 1000 CIDs
    for cid_chunk in normalized_cids.chunks(1000) {
        // Reset the builder and append the CID list
        builder.reset();
        let mut separated = builder.separated(", ");
        for cid in cid_chunk {
            separated.push_bind(cid);
        }
        separated.push_unseparated(");");

        builder
            .build()
            .execute(&database)
            .await
            .map_err(CreateSnapshotError::BlockAssociationFailed)?;
    }

    let resp_msg = serde_json::json!({ "id": snapshot_id });
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSnapshotError {
    #[error("no matching metadata for the current account")]
    NotFound,

    #[error("unable to locate requested metadata: {0}")]
    MetadataUnavailable(sqlx::Error),

    #[error("saving new snapshot association failed: {0}")]
    SaveFailed(sqlx::Error),

    #[error("associating the snapshot with the block cid failed: {0}")]
    BlockAssociationFailed(sqlx::Error),

    #[error("active cid list was in some way invalid: {0}")]
    InvalidInternalCid(cid::Error),
}

impl IntoResponse for CreateSnapshotError {
    fn into_response(self) -> Response {
        match &self {
            CreateSnapshotError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("encountered error creating snapshot: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
