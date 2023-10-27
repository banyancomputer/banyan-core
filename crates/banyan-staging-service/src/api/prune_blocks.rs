use std::ops::Deref;

use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use banyan_task::TaskLikeExt;
use cid::multibase::Base;
use cid::Cid;
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::PlatformAuthKey;
use crate::car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use crate::database::{BareId, SqlxError};
use crate::extractors::{CoreIdentity, Database};
use crate::tasks::{PruneBlock, PruneBlocksTask};

pub async fn handler(
    _ci: CoreIdentity,
    db: Database,
    Json(prune_blocks): Json<Vec<PruneBlock>>,
) -> Result<Response, PruneBlocksError> {
    let mut db = db.0;
    PruneBlocksTask::new(prune_blocks)
        .enqueue::<banyan_task::SqliteTaskStore>(&mut db)
        .await
        .map_err(PruneBlocksError::UnableToEnqueueTask)?;
    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksError {
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(banyan_task::TaskStoreError),
}

impl IntoResponse for PruneBlocksError {
    fn into_response(self) -> Response {
        use PruneBlocksError::*;
        match self {
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
