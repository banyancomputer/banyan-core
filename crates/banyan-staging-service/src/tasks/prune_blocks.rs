use std::collections::HashSet;

use async_trait::async_trait;
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Acquire};
use url::Url;
use uuid::Uuid;

use banyan_task::{CurrentTask, TaskLike};

// use crate::app::ServiceSigningKey;

pub type PruneBlocksTaskContext = SqlitePool;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksTaskError {
    #[error("the task encountered a sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PruneBlock {
    // TODO: this should be a CID? Based on how I'm using it right now, this is gauranteed to be a CID.
    pub normalized_cid: String,
    pub metadata_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PruneBlocksTask {
    prune_blocks: Vec<PruneBlock>,
}

impl PruneBlocksTask {
    pub fn new(prune_blocks: Vec<PruneBlock>) -> Self {
        Self {
            prune_blocks,
        }
    }
}

struct UniqueUploadsBlock {
    pub upload_id: String,
    pub block_id: String
}

#[async_trait]
impl TaskLike for PruneBlocksTask {
    const TASK_NAME: &'static str = "prune_blocks_task";

    type Error = PruneBlocksTaskError;
    type Context = PruneBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx
            .acquire()
            .await
            .map_err(PruneBlocksTaskError::SqlxError)?;
        let mut transaction = db_conn
            .begin()
            .await
            .map_err(PruneBlocksTaskError::SqlxError)?;

        for prune_block in &self.prune_blocks {
            // Get the block id
            let unique_uploads_block = sqlx::query_as!(
                UniqueUploadsBlock,
                r#"
                    SELECT u.id AS upload_id, blocks.id AS block_id
                    FROM uploads_blocks AS ub
                    JOIN blocks ON blocks.id = ub.block_id
                    JOIN uploads AS u ON u.id = ub.upload_id
                    WHERE blocks.cid = $1 AND u.metadata_id = $2;
                "#,
                prune_block.normalized_cid,
                prune_block.metadata_id
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(PruneBlocksTaskError::SqlxError)?;
        
            // Set the specifiec block as pruned
            sqlx::query!(
                r#"
                    UPDATE uploads_blocks SET pruned_at = CURRENT_TIMESTAMP
                    WHERE upload_id = $1 AND block_id = $2;
                "#,
                unique_uploads_block.upload_id,
                unique_uploads_block.block_id
            )
            .execute(&mut *transaction)
            .await
            .map_err(PruneBlocksTaskError::SqlxError)?; 
        }
        transaction.commit().await.map_err(PruneBlocksTaskError::SqlxError)?;
        Ok(())
    }
}
