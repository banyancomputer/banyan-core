use std::collections::HashSet;

use async_trait::async_trait;
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use banyan_task::{CurrentTask, TaskLike};

use crate::app::AppState;

pub type PruneBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksTaskError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PruneBlock {
    pub normalized_cid: String,
    pub metadata_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PruneBlocksTask {
    storage_host_id: Uuid,
    prune_blocks: Vec<PruneBlock>,
}

impl PruneBlocksTask {
    pub fn new(storage_host_id: Uuid, prune_blocks: Vec<PruneBlock>) -> Self {
        Self {
            storage_host_id,
            prune_blocks,
        }
    }
}

#[async_trait]
impl TaskLike for PruneBlocksTask {
    const TASK_NAME: &'static str = "prune_blocks_task";

    type Error = PruneBlocksTaskError;
    type Context = PruneBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx
            .database()
            .acquire()
            .await
            .map_err(PruneBlocksTaskError::SqlxError)?;
        let auth_key = ctx.secrets().service_key();

        // Determine where to send the prune list
        let storage_host_id = self.storage_host_id.to_string();
        let storage_host_url = sqlx::query_scalar!(
            "SELECT url FROM storage_hosts WHERE id = $1;",
            storage_host_id
        )
        .fetch_one(&mut *db_conn)
        .await
        .map_err(PruneBlocksTaskError::SqlxError)?;
        let storage_host_url = Url::parse(&storage_host_url)
            .map_err(|_| PruneBlocksTaskError::SqlxError(sqlx::Error::RowNotFound))?;
        let storage_host_url = storage_host_url
            .join("/api/v1/core/prune")
            .map_err(|_| PruneBlocksTaskError::SqlxError(sqlx::Error::RowNotFound))?;

        // Construct the client to handle the prune request
        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .map_err(PruneBlocksTaskError::ReqwestError)?;
        let mut claims = Claims::create(Duration::from_secs(60))
            .with_audiences(HashSet::from_strings(&["banyan-platform"]))
            .with_subject("banyan-core")
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));
        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());
        let bearer_token = auth_key.sign(claims).unwrap();

        // Send the request and handle the response
        let request = client
            .post(storage_host_url.clone())
            .json(&self.prune_blocks)
            .bearer_auth(bearer_token);
        let response = request
            .send()
            .await
            .map_err(PruneBlocksTaskError::ReqwestError)?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(PruneBlocksTaskError::HttpError(
                response.status(),
                storage_host_url,
            ))
        }
    }
}
