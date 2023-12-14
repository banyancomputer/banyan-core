use std::collections::BTreeSet;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cid::multibase::Base;
use cid::Cid;
use itertools::Itertools;
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateSnapshotRequest>,
) -> Result<Response, CreateSnapshotError> {
    let database = state.database();
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    let user_id = user_identity.id().to_string();
    let owned_metadata_id = sqlx::query_scalar!(
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

    let snapshot_id = sqlx::query_scalar!(
        r#"INSERT INTO snapshots (metadata_id, state)
               VALUES ($1, 'pending')
               RETURNING id;"#,
        owned_metadata_id,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateSnapshotError::SaveFailed)?;

    // Grab all the cids
    let normalized_cids = request
        .active_cids
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

    // Batched writes of 1k
    for cid_chunk in &normalized_cids.into_iter().chunks(1000) {
        // Build a query for all 1k cids
        let mut builder =
            sqlx::QueryBuilder::new("INSERT INTO snapshot_block_locations(snapshot_id, block_id) ");
        builder.push_values(cid_chunk, |mut build, cid| {
            build.push_bind(snapshot_id.clone()).push_bind(cid);
        });
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

#[derive(Deserialize)]
pub struct CreateSnapshotRequest {
    pub bucket_id: Uuid,
    pub metadata_id: Uuid,
    pub active_cids: BTreeSet<Cid>,
}
