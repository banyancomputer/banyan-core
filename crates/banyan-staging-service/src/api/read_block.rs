use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::database::models::BlockDetails;
use crate::database::Database;
use crate::extractors::BlockReader;
use crate::upload_store::{ObjectStore, UploadStore};

pub async fn handler(
    State(state): State<AppState>,
    client: BlockReader,
    store: UploadStore,
    Path(cid): Path<String>,
) -> Result<Response, BlockReadError> {
    let db = state.database();
    let cid = cid::Cid::try_from(cid).map_err(BlockReadError::InvalidCid)?;
    let normalized_cid = cid
        .to_string_of_base(cid::multibase::Base::Base64Url)
        .expect("parsed cid to unparse");

    tracing::warn!("NORMAL CID IN READ: {normalized_cid}");

    let block_details = block_from_normalized_cid(&db, &normalized_cid).await?;
    if !client.can_read_block(&block_details) {
        return Err(BlockReadError::NotBlockOwner);
    }

    let byte_start = block_details.byte_offset as usize;
    let byte_end = byte_start + (block_details.length as usize);
    let byte_range = byte_start..byte_end;

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
        byte_range.len().to_string().as_str().parse().unwrap(),
    );

    // this isn't ideal as we have to load the entire block from memory, object_store does support
    // passing in the byte range using GetOptions to the get_opts method on the ObjectStore trait,
    // however data in the "File" type explicitly ignores this range which is incredibly
    // frustrating...

    // If this is one of the deprecated CAR file Uploads
    if block_details.blocks_path.to_lowercase().ends_with(".car") {
        let data = store
            .get_range(
                &object_store::path::Path::from(block_details.blocks_path.as_str()),
                byte_range,
            )
            .await
            .map_err(BlockReadError::RetrievalFailed)?;
        Ok((StatusCode::OK, headers, data).into_response())
    }
    // If this block exists in its own dedicated file
    else {
        let object_path = object_store::path::Path::from(format!(
            "{}/{}.block",
            block_details.blocks_path, normalized_cid
        ));
        let data = store
            .get(&object_path)
            .await
            .map_err(BlockReadError::RetrievalFailed)?
            .bytes()
            .await
            .map_err(BlockReadError::RetrievalFailed)?;
        Ok((StatusCode::OK, headers, data).into_response())
    }
}

pub async fn block_from_normalized_cid(
    db: &Database,
    normalized_cid: &str,
) -> Result<BlockDetails, BlockReadError> {
    // let maybe_block_id: Option<String> = sqlx::query_scalar(
    //     r#"
    //     SELECT
    //             blocks.id as id,
    //             blocks.data_length as length,
    //             clients.platform_id AS platform_id,
    //             uploads.blocks_path AS blocks_path,
    //     FROM blocks
    //         JOIN uploads
    //     WHERE blocks.cid = $1;
    //     "#
    // )
    // .bind(normalized_cid)
    // .fetch_one(db)
    // .await
    // .map_err(BlockReadError::DbFailure)?;

    // let block_id = maybe_block_id.ok_or(BlockReadError::UnknownBlock)?;

    let maybe_block_id: Option<BlockDetails> = sqlx::query_as(
        r#"
            SELECT
                    blocks.id AS id,
                    clients.platform_id AS platform_id,
                    uploads.blocks_path AS blocks_path,
                    uploads_blocks.byte_offset AS byte_offset,
                    blocks.data_length AS length
                FROM blocks
                    JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
                    JOIN uploads ON uploads_blocks.upload_id = uploads.id
                    JOIN clients ON uploads.client_id = clients.id
                WHERE blocks.cid = $1;
            "#,
    )
    .bind(normalized_cid)
    .fetch_optional(db)
    .await
    .map_err(BlockReadError::DbFailure)?;
    match maybe_block_id {
        Some(id) => Ok(id),
        None => Err(BlockReadError::UnknownBlock),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockReadError {
    #[error("internal database error occurred")]
    DbFailure(sqlx::Error),

    #[error("request for invalid CID rejected")]
    InvalidCid(cid::Error),

    #[error("authenticated user requested block not owned by them")]
    NotBlockOwner,

    #[error("unable to pull block that should exist")]
    RetrievalFailed(object_store::Error),

    #[error("requested block was not in our database")]
    UnknownBlock,
}

impl IntoResponse for BlockReadError {
    fn into_response(self) -> Response {
        use BlockReadError::*;

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
