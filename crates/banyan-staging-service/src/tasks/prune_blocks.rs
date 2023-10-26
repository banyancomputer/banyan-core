use std::collections::HashSet;

use async_trait::async_trait;
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, SqlitePool};
use url::Url;
use uuid::Uuid;

use banyan_task::{CurrentTask, TaskLike};

use crate::app::{PlatformAuthKey, State};

pub type PruneBlocksTaskContext = State;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksTaskError {
    #[error("the task encountered a sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("the task encountered a reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("the task encountered a jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("the task encountered a non success response")]
    NonSuccessResponse(http::StatusCode),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PruneBlock {
    pub normalized_cid: String,
    pub metadata_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PruneBlocksTask {
    prune_blocks: Vec<PruneBlock>,
}

impl PruneBlocksTask {
    pub fn new(prune_blocks: Vec<PruneBlock>) -> Self {
        Self { prune_blocks }
    }
}

struct UniqueUploadsBlock {
    pub upload_id: String,
    pub block_id: String,
}

#[async_trait]
impl TaskLike for PruneBlocksTask {
    const TASK_NAME: &'static str = "prune_blocks_task";

    type Error = PruneBlocksTaskError;
    type Context = PruneBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let auth_key = ctx.platform_auth_key();
        let mut db_conn = ctx
            .database()
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

        report_pruned_blocks(&auth_key, &self.prune_blocks).await?;

        transaction
            .commit()
            .await
            .map_err(PruneBlocksTaskError::SqlxError)?;
        Ok(())
    }
}

async fn report_pruned_blocks(
    auth_key: &PlatformAuthKey,
    prune_blocks: &Vec<PruneBlock>,
) -> Result<(), PruneBlocksTaskError> {
    let mut default_headers = HeaderMap::new();
    default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let client = Client::builder()
        .default_headers(default_headers)
        .build()
        .unwrap();

    let report_endpoint = auth_key
        .base_url()
        .join("/hooks/storage/prune".to_string().as_str())
        .unwrap();

    let mut claims = Claims::create(Duration::from_secs(60))
        .with_audiences(HashSet::from_strings(&["banyan-platform"]))
        .with_subject("banyan-staging")
        .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

    claims.create_nonce();
    claims.issued_at = Some(Clock::now_since_epoch());

    let bearer_token = auth_key.sign(claims).unwrap();

    let request = client
        .post(report_endpoint)
        .json(&prune_blocks)
        .bearer_auth(bearer_token);

    let response = request
        .send()
        .await
        .map_err(PruneBlocksTaskError::ReqwestError)?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(PruneBlocksTaskError::NonSuccessResponse(response.status()))
    }
}