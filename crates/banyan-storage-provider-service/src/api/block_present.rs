use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::extractors::BlockReader;

pub async fn handler(
    _: BlockReader,
    State(state): State<AppState>,
    Json(cids): Json<Vec<String>>,
) -> Result<Response, BlockPresentError> {
    let db = state.database();

    let mut prune_builder = sqlx::QueryBuilder::new("SELECT cid FROM blocks WHERE cid IN(");
    let mut separated = prune_builder.separated(", ");
    for bid in cids.iter() {
        separated.push_bind(bid);
    }
    prune_builder.push(");");

    let block_details: Vec<String> = prune_builder.build_query_scalar().fetch_all(&db).await?;

    Ok((StatusCode::OK, Json(block_details)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BlockPresentError {
    #[error("internal database error occurred")]
    DbFailure(#[from] sqlx::Error),

    #[error("request for invalid CID rejected")]
    InvalidCid(String),
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
            InvalidCid(cid) => {
                tracing::warn!("invalid CID: {}", cid);
                let err_msg = serde_json::json!({ "msg": "blocks not found" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
