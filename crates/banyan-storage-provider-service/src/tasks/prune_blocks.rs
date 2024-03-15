use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;

pub type PruneBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksTaskError {
    #[error("sql error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::Error),

    #[error("http error: {0} response from {1}")]
    Http(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct PruneBlocksTask {
    prune_blocks: Vec<String>,
}

impl PruneBlocksTask {
    pub fn new(prune_blocks: Vec<String>) -> Self {
        Self { prune_blocks }
    }
}

#[async_trait]
impl TaskLike for PruneBlocksTask {
    const TASK_NAME: &'static str = "prune_blocks_task";

    type Error = PruneBlocksTaskError;
    type Context = PruneBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut conn = ctx.database().acquire().await?;

        let mut block_id_query = sqlx::QueryBuilder::new("SELECT id FROM blocks WHERE cid IN (");

        let mut cid_iterator = self.prune_blocks.iter().peekable();
        while let Some(cid) = cid_iterator.next() {
            block_id_query.push_bind(cid);

            if cid_iterator.peek().is_some() {
                block_id_query.push(", ");
            }
        }

        block_id_query.push(");");
        let block_ids: Vec<String> = block_id_query
            .build_query_scalar()
            .persistent(false)
            .fetch_all(&mut *conn)
            .await?;

        let mut prune_builder = sqlx::QueryBuilder::new(
            r#"UPDATE uploads_blocks SET pruned_at = CURRENT_TIMESTAMP
                   WHERE pruned_at IS NULL AND block_id IN ("#,
        );

        let mut block_id_iterator = block_ids.iter().peekable();
        while let Some(bid) = block_id_iterator.next() {
            prune_builder.push_bind(bid);

            if block_id_iterator.peek().is_some() {
                prune_builder.push(", ");
            }
        }

        prune_builder.push(");");
        let prune_result = prune_builder.build().execute(&mut *conn).await?;

        report_pruned_blocks(&ctx, &self.prune_blocks).await?;

        tracing::info!(
            pruned_blocks = prune_result.rows_affected(),
            "blocked pruned"
        );

        Ok(())
    }
}

async fn report_pruned_blocks(
    ctx: &PruneBlocksTaskContext,
    prune_blocks: &Vec<String>,
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

    let report_endpoint = platform_hostname.join("/hooks/storage/prune").unwrap();

    let mut claims = Claims::create(Duration::from_secs(60))
        .with_audiences(HashSet::from_strings(&[platform_name]))
        .with_subject(service_name)
        .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

    claims.create_nonce();
    claims.issued_at = Some(Clock::now_since_epoch());

    let bearer_token = service_signing_key.sign(claims)?;

    let request = client
        .post(report_endpoint.clone())
        .json(&prune_blocks)
        .bearer_auth(bearer_token);

    let response = request.send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(PruneBlocksTaskError::Http(
            response.status(),
            report_endpoint,
        ))
    }
}
