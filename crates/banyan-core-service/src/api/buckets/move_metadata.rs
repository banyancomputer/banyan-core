use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jwt_simple::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::buckets::metadata::rounded_storage_authorization;
use crate::app::AppState;
use crate::auth::storage_ticket::StorageTicketBuilder;
use crate::database::models::{
    AuthorizedAmounts, Bucket, Metadata, NewStorageGrant, PendingExpiration, SelectedStorageHost,
    StorageHost,
};
use crate::database::DatabaseConnection;
use crate::extractors::StorageProviderIdentity;
use crate::utils;

pub async fn handler(
    // TODO: in reality it should be staging service only
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<Uuid>,
    Json(request): Json<MoveMetadataRequest>,
) -> Result<Response, MoveMetadataError> {
    let span = tracing::info_span!(
        "move_metadata",
        storage_provider = %storage_provider.id,
        metadata_id = %metadata_id,
    );
    let _guard = span.enter();

    let metadata_id = metadata_id.to_string();
    let database = state.database();
    let mut conn = database.begin().await?;

    // check that the calling storage host does indeed store the specific upload
    if !Metadata::storage_host_owns_metadata(&mut conn, &metadata_id, &storage_provider.id).await? {
        tracing::warn!("attempted to move files by storage host that does own metadata");
        let err_msg = serde_json::json!({"msg": "metadata not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    let bucket_id = Metadata::get_bucket_id(&mut conn, &metadata_id).await?;
    // user is making changes. try moving the bucket contents later
    if Bucket::is_change_in_progress(&mut conn, &bucket_id).await? {
        tracing::warn!("attempted to move files from bucket while other write was in progress");
        let err_msg = serde_json::json!({"msg": "waiting for other upload to complete"});
        return Ok((StatusCode::CONFLICT, Json(err_msg)).into_response());
    }

    let normalized_cids: Vec<_> = match request
        .previous_cids
        .iter()
        .map(String::as_str)
        .map(utils::normalize_cid)
        .collect()
    {
        Ok(nc) => nc,
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "request data included invalid CID in deleted block list"});
            return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
        }
    };

    // pull all the normalized_cids from the old upload using the metadata_id and expire them
    PendingExpiration::record_pending_block_expirations(
        &mut conn,
        &bucket_id,
        &metadata_id,
        &normalized_cids,
    )
    .await?;

    // TODO: YES/NO on the below?
    // Checkpoint the upload to the database so we can track failures, and perform any necessary
    // clean up behind the scenes. The upload itself will also dwarf the rest of the time of this
    // request, limiting the time in those transactions is a good idea.
    // conn.commit().await?;

    let needed_capacity = request.needed_capacity;

    let new_storage_host = match SelectedStorageHost::select_for_capacity(
        &mut conn,
        needed_capacity,
        Some(&storage_provider.id),
    )
    .await?
    {
        Some(sh) => sh,
        None => {
            tracing::warn!(
                needed_capacity,
                "unable to locate host with sufficient capacity"
            );
            let err_msg = serde_json::json!({"msg": ""});
            return Ok((StatusCode::INSUFFICIENT_STORAGE, Json(err_msg)).into_response());
        }
    };

    let storage_grant_id = ensure_grant_space(
        &mut conn,
        &new_storage_host,
        &bucket_id,
        needed_capacity,
        &metadata_id,
    )
    .await?;

    let mut ticket_builder = StorageTicketBuilder::new(new_storage_host.name.clone());
    ticket_builder.add_audience(storage_provider.name.to_string());
    ticket_builder.add_authorization(
        storage_grant_id,
        new_storage_host.url.clone(),
        needed_capacity,
    );

    let claim = ticket_builder.build();

    let storage_authorization = match state.secrets().service_key().sign(claim) {
        Ok(t) => t,
        Err(err) => {
            tracing::error!("failed to sign storage authorization: {err}");
            let err_msg = serde_json::json!({"msg": "authorization delegation unavailable"});
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response());
        }
    };

    conn.commit().await?;

    let response = serde_json::json!({
        "storage_host": new_storage_host.url,
        "storage_authorization": storage_authorization,
    });

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum MoveMetadataError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),

    #[error("the request was badly formatted: {0}")]
    InvalidMultipart(#[from] multer::Error),
}

impl IntoResponse for MoveMetadataError {
    fn into_response(self) -> Response {
        tracing::error!("internal error handling metadata upload: {self}");
        let err_msg = serde_json::json!({"msg": "a backend service issue encountered an error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}

#[derive(Deserialize)]
pub struct MoveMetadataRequest {
    needed_capacity: i64,
    // block IDs stored on the old host, to be deleted and then moved to a new host
    previous_cids: Vec<String>,
}

pub async fn ensure_grant_space(
    conn: &mut DatabaseConnection,
    new_storage_host: &SelectedStorageHost,
    bucket_id: &str,
    required_space: i64,
    metadata_id: &str,
) -> Result<String, sqlx::Error> {
    let user_id = Metadata::get_user_id(conn, metadata_id).await?;
    let user_report = StorageHost::user_report(conn, &new_storage_host.id, &user_id).await?;
    let authorized_amounts = AuthorizedAmounts::lookup(conn, &user_id, &bucket_id).await?;
    let existing_grant = authorized_amounts
        .into_iter()
        .filter(|auth_details| auth_details.authorized_amount >= required_space)
        .next();

    let storage_grant_id = match existing_grant {
        Some(grant) => grant.storage_grant_id,
        None => {
            let new_authorized_capacity =
                rounded_storage_authorization(&user_report, required_space);
            NewStorageGrant {
                storage_host_id: &new_storage_host.id,
                user_id: &user_id,
                authorized_amount: new_authorized_capacity,
            }
            .save(conn)
            .await?
        }
    };
    Ok(storage_grant_id)
}
