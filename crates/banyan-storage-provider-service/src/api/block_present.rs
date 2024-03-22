use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cid::Cid;

use crate::app::AppState;
use crate::database::Database;
use crate::extractors::BlockReader;

pub async fn handler(
    _: BlockReader,
    State(state): State<AppState>,
    Json(cids): Json<Vec<String>>,
) -> Result<Response, BlockPresentError> {
    let db = state.database();
    let mut normalized_cid = Vec::new();
    for cid in &cids {
        let cid_str = Cid::try_from(cid.as_str())
            .map_err(BlockPresentError::InvalidCid)
            .and_then(|cid| {
                cid.to_string_of_base(cid::multibase::Base::Base64Url)
                    .map_err(BlockPresentError::InvalidCid)
            })?;
        normalized_cid.push(cid_str);
    }

    let block_details = present_cids(&db, normalized_cid).await?;

    Ok((StatusCode::OK, Json(block_details)).into_response())
}

pub async fn present_cids(
    database: &Database,
    normalized_cids: Vec<String>,
) -> Result<Vec<String>, sqlx::Error> {
    let mut prune_builder = sqlx::QueryBuilder::new("SELECT * FROM blocks WHERE cid IN(");

    let mut block_id_iterator = normalized_cids.iter().peekable();
    while let Some(bid) = block_id_iterator.next() {
        prune_builder.push_bind(bid);

        if block_id_iterator.peek().is_some() {
            prune_builder.push(", ");
        }
    }
    prune_builder.push(");");

    let res = prune_builder
        .build_query_scalar()
        .fetch_all(database)
        .await?;
    Ok(res)
}

#[derive(Debug, thiserror::Error)]
pub enum BlockPresentError {
    #[error("internal database error occurred")]
    DbFailure(#[from] sqlx::Error),

    #[error("request for invalid CID rejected")]
    InvalidCid(#[from] cid::Error),
}

impl IntoResponse for BlockPresentError {
    fn into_response(self) -> Response {
        use BlockPresentError::*;

        match self {
            DbFailure(err) => {
                tracing::warn!("db failure looking up block: {}", err);
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            InvalidCid(err) => {
                tracing::warn!("invalid CID: {}", err);
                let err_msg = serde_json::json!({ "msg": "blocks not found" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
