use axum::extract::{BodyStream, State};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, TypedHeader};
use banyan_object_store::{ObjectStore, ObjectStorePath};
use banyan_task::TaskLikeExt;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::upload::db::{complete_upload, write_block_to_tables};
use crate::app::AppState;
use crate::clients::{CoreServiceClient, ReportRedistributionRequest};
use crate::database::models::Upload;
use crate::database::DatabaseConnection;
use crate::extractors::PlatformIdentity;
use crate::tasks::ReportRedistributionTask;

#[derive(Deserialize, Serialize)]
pub struct BlockUploadRequest {
    cid: String,
    details: BlockUploadDetails,
}

#[derive(Serialize, Deserialize)]
pub struct BlockUploadDetails {
    pub replication: bool,
    pub completed: bool,
    pub upload_id: String,
    pub grant_id: Uuid,
}
pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    store: ObjectStore,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, BlocksUploadError> {
    let db = state.database();
    let mut conn = db.acquire().await?;

    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct).unwrap();
    let constraints = multer::Constraints::new().allowed_fields(vec!["request-data", "block"]);

    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);
    let mut total_size = 0;

    // Grab the request object
    let request: BlockUploadRequest = multipart
        .next_field()
        .await
        .map_err(BlocksUploadError::RequestFieldUnavailable)?
        .ok_or(BlocksUploadError::RequestFieldMissing)?
        .json()
        .await
        .map_err(BlocksUploadError::InvalidRequestData)?;

    let upload = Upload::by_id(&mut conn, &request.details.upload_id).await?;

    if upload.state == "complete" {
        return Err(BlocksUploadError::UploadIsComplete);
    }

    let mut conn = db.acquire().await?;
    // While there are still block fields encoded
    while let Some(block_field) = multipart
        .next_field()
        .await
        .map_err(BlocksUploadError::DataFieldUnavailable)?
    {
        // Grab all of the block data from this request part
        let block: Bytes = block_field
            .bytes()
            .await
            .map_err(BlocksUploadError::DataFieldUnavailable)?;

        if !crate::utils::is_valid_cid(&request.cid) {
            return Err(BlocksUploadError::InvalidCid)?;
        }

        // Write this block to the tables
        write_block_to_tables(&mut conn, &upload.id, &request.cid, block.len() as i64).await?;
        total_size += block.len();

        // Write the bytes to the expected location
        let location =
            ObjectStorePath::from(format!("{}/{}.bin", upload.base_path, request.cid).as_str());
        store
            .put(&location, block)
            .await
            .map_err(BlocksUploadError::ObjectStore)?;
    }

    // If we've just finished off the upload, complete and report it
    if request.details.completed {
        complete_upload(&mut conn, total_size as i64, "", &upload.id).await?;

        report_complete_redistribution(
            &mut conn,
            request.details.grant_id,
            &upload.metadata_id,
            &upload.id,
            total_size,
            request.details.replication,
        )
        .await?;
    }

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BlocksUploadError {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("request's data payload was malformed")]
    InvalidRequestData(multer::Error),

    #[error("Data in request mismatched attached CID")]
    MismatchedCid((String, String)),

    #[error("the provided CID is invalid")]
    InvalidCid,

    #[error("failed to acquire data field from body")]
    DataFieldUnavailable(multer::Error),

    #[error("tried to write to a completed upload")]
    UploadIsComplete,

    #[error("failed to acquire request field from body")]
    RequestFieldUnavailable(multer::Error),

    #[error("we expected a request field but received nothing")]
    RequestFieldMissing,

    #[error("failed to write to storage backend")]
    ObjectStore(banyan_object_store::ObjectStoreError),

    #[error("account is not authorized to store {0} bytes, {1} bytes are still authorized")]
    InsufficientAuthorizedStorage(u64, u64),
}

impl IntoResponse for BlocksUploadError {
    fn into_response(self) -> Response {
        use BlocksUploadError::*;
        let default_err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
        let default_response =
            (StatusCode::INTERNAL_SERVER_ERROR, Json(default_err_msg)).into_response();
        match self {
            DatabaseError(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            DataFieldUnavailable(_)
            | InvalidRequestData(_)
            | InvalidCid
            | RequestFieldUnavailable(_)
            | MismatchedCid(_)
            | RequestFieldMissing => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            InsufficientAuthorizedStorage(requested_bytes, remaining_bytes) => {
                tracing::warn!(upload_size = ?requested_bytes, remaining_storage = ?remaining_bytes, "user doesn't have sufficient storage capacity remaining");
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response()
            }
            ObjectStore(err) => {
                tracing::error!("writing car file failed: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            UploadIsComplete => {
                tracing::warn!("client is trying to write more data to a completed upload");
                default_response
            }
        }
    }
}

pub async fn report_complete_redistribution(
    conn: &mut DatabaseConnection,
    grant_id: Uuid,
    metadata_id: &str,
    upload_id: &str,
    total_size: i64,
    replication: bool,
) -> Result<(), sqlx::Error> {
    let all_cids: Vec<String> = sqlx::query_scalar!(
        r#"
            SELECT blocks.cid
            FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            WHERE uploads.id = $1;
        "#,
        upload_id
    )
    .fetch_all(&mut *conn)
    .await?;

    let all_cids = all_cids
        .into_iter()
        .map(|cid_string| Cid::from_str(&cid_string).unwrap())
        .collect::<Vec<Cid>>();

    ReportRedistributionTask::new(grant_id, metadata_id, &all_cids, total_size, replication)
        .enqueue::<banyan_task::SqliteTaskStore>(&mut *conn)
        .await
        .unwrap();

    Ok(())
}
