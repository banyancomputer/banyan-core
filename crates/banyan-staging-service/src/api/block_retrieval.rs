use axum::Json;
use axum::extract::Path;
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use object_store::ObjectStore;
use uuid::Uuid;
use cid::Cid;

use crate::database::{DbError, Executor};
use crate::extractors::{AuthenticatedClient, Database, UploadStore};

pub async fn handler(
    db: Database,
    _client: AuthenticatedClient,
    _store: UploadStore,
    Path(cid): Path<String>,
) -> Result<Response, BlockRetrievalError> {
    let cid = cid::Cid::try_from(cid).map_err(BlockRetrievalError::InvalidCid)?;

    let normalized_cid = cid
        .to_string_of_base(cid::multibase::Base::Base64Url)
        .expect("parsed cid to unparse");

    let _block = block_from_normalized_cid(&db, &normalized_cid).await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(sqlx::FromRow)]
pub struct BlockDetails {
    id: String,
    platform_id: String,

    file_path: String,
    byte_offset: i64,
    length: i64,
}

pub async fn block_from_normalized_cid(
    db: &Database,
    normalized_cid: &str,
) -> Result<BlockDetails, BlockRetrievalError> {
    match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let maybe_block_id: Option<BlockDetails> = sqlx::query_as(r#"
                SELECT
                        blocks.id AS id,
                        clients.platform_id AS platform_id,
                        uploads.file_path AS file_path,
                        uploads_blocks.byte_offset AS byte_offset,
                        blocks.data_length AS length,
                    FROM blocks
                        JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
                        JOIN uploads ON uploads_blocks.upload_id = uploads.id
                        JOIN clients ON uploads.client_id = clients.id
                    WHERE b.cid = $1;
            "#)
            .bind(normalized_cid)
            .fetch_optional(conn)
            .await
            .map_err(postgres::map_sqlx_error)
            .map_err(BlockRetrievalError::DbFailure)?;

            match maybe_block_id {
                Some(id) => Ok(id),
                None => Err(BlockRetrievalError::UnknownBlock),
            }
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let maybe_block_id: Option<BlockDetails> = sqlx::query_as(r#"
                SELECT
                        blocks.id AS id,
                        clients.platform_id AS platform_id,
                        uploads.file_path AS file_path,
                        uploads_blocks.byte_offset AS byte_offset,
                        blocks.data_length AS length,
                    FROM blocks
                        JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
                        JOIN uploads ON uploads_blocks.upload_id = uploads.id
                        JOIN clients ON uploads.client_id = clients.id
                    WHERE b.cid = $1;
            "#)
            .bind(normalized_cid)
            .fetch_optional(conn)
            .await
            .map_err(sqlite::map_sqlx_error)
            .map_err(BlockRetrievalError::DbFailure)?;

            match maybe_block_id {
                Some(id) => Ok(id),
                None => Err(BlockRetrievalError::UnknownBlock),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockRetrievalError {
    #[error("internal database error occurred")]
    DbFailure(DbError),

    #[error("request for invalid CID rejected")]
    InvalidCid(cid::Error),

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
            InvalidCid(err) => {
                tracing::warn!("client attempted authenticated upload with invalid CID: {err}");
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            UnknownBlock => {
                let err_msg = serde_json::json!({ "msg": format!("unknown block") });
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}
