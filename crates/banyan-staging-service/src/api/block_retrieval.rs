use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_object_store::{ObjectStore, ObjectStoreError, ObjectStorePath};

use crate::app::AppState;
use crate::database::models::BlockDetails;
use crate::database::Database;
use crate::extractors::BlockReader;

pub async fn handler(
    State(state): State<AppState>,
    client: BlockReader,
    store: ObjectStore,
    Path(cid): Path<String>,
) -> Result<Response, BlockRetrievalError> {
    let db = state.database();
    let cid = cid::Cid::try_from(cid).map_err(BlockRetrievalError::InvalidCid)?;
    let normalized_cid = cid
        .to_string_of_base(cid::multibase::Base::Base64Url)
        .expect("parsed cid to unparse");

    let block_details = block_from_normalized_cid(&db, &normalized_cid).await?;
    if !client.can_read_block(&block_details) {
        return Err(BlockRetrievalError::NotBlockOwner);
    }

    let mut headers = axum::http::HeaderMap::new();

    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        "attachment; filename=\"{normalized_cid}.bin\""
            .parse()
            .unwrap(),
    );
    headers.insert(
        axum::http::header::CONTENT_LENGTH,
        block_details.length.to_string().as_str().parse().unwrap(),
    );

    // this isn't ideal as we have to load the entire block from memory, object_store does support
    // passing in the byte range using GetOptions to the get_opts method on the ObjectStore trait,
    // however data in the "File" type explicitly ignores this range which is incredibly
    // frustrating...

    // If the car_offset is valid, expect to find a CAR file in the object store
    if let Some(car_offset) = block_details.car_offset {
        let byte_start = car_offset as usize;
        let byte_end = byte_start + (block_details.length as usize);
        let byte_range = byte_start..byte_end;
        // In the case of CAR files, the base path is already a complete reference to file location
        let object_path = ObjectStorePath::from(block_details.base_path);
        let data = store
            .get_range(&object_path, byte_range)
            .await
            .map_err(BlockRetrievalError::RetrievalFailed)?;
        Ok((StatusCode::OK, headers, data).into_response())
    }
    // If the car_offset is null (no CAR file in the object store)
    else {
        // First try with the new version
        let object_path = ObjectStorePath::from(format!(
            "{}/{}.bin",
            block_details.base_path, normalized_cid
        ));

        // Get the data from the expected block location
        let data = store
            .get(&object_path)
            .await
            .map_err(BlockRetrievalError::RetrievalFailed)?
            .bytes()
            .await
            .map_err(BlockRetrievalError::RetrievalFailed)?;
        Ok((StatusCode::OK, headers, data).into_response())
    }
}

pub async fn block_from_normalized_cid(
    database: &Database,
    normalized_cid: &str,
) -> Result<BlockDetails, BlockRetrievalError> {
    let maybe_block_id: Option<BlockDetails> = sqlx::query_as(
        r#"
        SELECT
            blocks.id AS id,
                blocks.data_length AS length,
                uploads_blocks.car_offset AS car_offset,
                uploads.base_path AS base_path,
                clients.platform_id AS platform_id
        FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            JOIN clients ON uploads.client_id = clients.id
            WHERE blocks.cid = $1;
        "#,
    )
    .bind(normalized_cid)
    .fetch_optional(database)
    .await
    .map_err(BlockRetrievalError::DbFailure)?;
    match maybe_block_id {
        Some(id) => Ok(id),
        None => Err(BlockRetrievalError::UnknownBlock),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockRetrievalError {
    #[error("internal database error occurred")]
    DbFailure(sqlx::Error),

    #[error("request for invalid CID rejected")]
    InvalidCid(cid::Error),

    #[error("authenticated user requested block not owned by them")]
    NotBlockOwner,

    #[error("unable to pull block that should exist")]
    RetrievalFailed(#[from] ObjectStoreError),

    #[error("requested block was not in our database")]
    UnknownBlock,
}

impl IntoResponse for BlockRetrievalError {
    fn into_response(self) -> Response {
        use BlockRetrievalError::*;

        match &self {
            DbFailure(err) => {
                tracing::warn!("db failure looking up block: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            RetrievalFailed(err) => {
                tracing::error!("{self}: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            InvalidCid(err) => {
                tracing::warn!("client attempted authenticated upload with invalid CID: {err}");
                let err_msg = serde_json::json!({ "msg": format!("block not found") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            NotBlockOwner => {
                tracing::warn!("client attempted to access block that wasn't theirs");
                let err_msg = serde_json::json!({ "msg": format!("block not found") });
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            UnknownBlock => {
                let err_msg = serde_json::json!({ "msg": format!("block not found") });
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}
