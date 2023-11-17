use std::collections::HashSet;

use async_trait::async_trait;
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::Acquire;

use uuid::Uuid;

use banyan_task::{CurrentTask, TaskLike};

use crate::app::AppState;

pub type PruneBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksTaskError {
    #[error("the task encountered a sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
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
        // let service_signing_key = ctx.secrets().service_signing_key();
        // let service_name = ctx.service_name();
        // let platform_hostname = ctx.platform_hostname();
        let mut db_conn = ctx
            .database()
            .acquire()
            .await
            .map_err(PruneBlocksTaskError::DatabaseError)?;
        let mut transaction = db_conn
            .begin()
            .await
            .map_err(PruneBlocksTaskError::DatabaseError)?;

        for prune_block in &self.prune_blocks {
            let metadata_id = prune_block.metadata_id.to_string();
            // Get the block id
            let unique_uploads_block = sqlx::query_as!(
                UniqueUploadsBlock,
                r#"
                    SELECT u.id AS upload_id, b.id AS block_id
                    FROM uploads_blocks AS ub
                    JOIN blocks AS b ON b.id = ub.block_id
                    JOIN uploads AS u ON u.id = ub.upload_id
                    WHERE b.cid = $1 AND u.metadata_id = $2;
                "#,
                prune_block.normalized_cid,
                metadata_id
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(PruneBlocksTaskError::DatabaseError)?;

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
            .map_err(PruneBlocksTaskError::DatabaseError)?;
        }

        report_pruned_blocks(&ctx, &self.prune_blocks).await?;

        transaction
            .commit()
            .await
            .map_err(PruneBlocksTaskError::DatabaseError)?;
        Ok(())
    }
}

async fn report_pruned_blocks(
    ctx: &PruneBlocksTaskContext,
    prune_blocks: &Vec<PruneBlock>,
) -> Result<(), PruneBlocksTaskError> {
    let service_signing_key = ctx.secrets().service_signing_key();
    let service_name = ctx.service_name();
    let platform_name = ctx.platform_name();
    let platform_hostname = ctx.platform_hostname();

    let mut default_headers = HeaderMap::new();
    default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let client = Client::builder()
        .default_headers(default_headers)
        .build()
        .unwrap();

    let report_endpoint = platform_hostname
        .join("/hooks/storage/prune".to_string().as_str())
        .unwrap();

    let mut claims = Claims::create(Duration::from_secs(60))
        .with_audiences(HashSet::from_strings(&[platform_name]))
        .with_subject(service_name)
        .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

    claims.create_nonce();
    claims.issued_at = Some(Clock::now_since_epoch());

    let bearer_token = service_signing_key
        .sign(claims)
        .map_err(PruneBlocksTaskError::JwtError)?;

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
