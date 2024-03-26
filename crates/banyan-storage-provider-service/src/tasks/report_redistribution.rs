use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::clients::{CoreServiceClient, CoreServiceError, ReportRedistributionRequest};

pub type ReportRedistributionTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportRedistributionTaskError {
    #[error("reqwest error: {0}")]
    CoreServiceError(#[from] CoreServiceError),

    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("invalid cid")]
    InvalidCid,
}

#[derive(Deserialize, Serialize)]
pub struct ReportRedistributionTask {
    grant_id: Uuid,
    metadata_id: String,
    cids: Vec<String>,
    data_size: u64,
}

impl ReportRedistributionTask {
    pub fn new(grant_id: Uuid, metadata_id: &str, cids: &[String], data_size: u64) -> Self {
        Self {
            grant_id,
            metadata_id: String::from(metadata_id),
            cids: cids.to_vec(),
            data_size,
        }
    }
}

#[async_trait]
impl TaskLike for ReportRedistributionTask {
    const TASK_NAME: &'static str = "report_redistribution_task";

    type Error = ReportRedistributionTaskError;
    type Context = ReportRedistributionTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let grant_id = self.grant_id.to_string();
        let data_size = self.data_size;

        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );

        client
            .report_distribution_complete(
                &self.metadata_id,
                ReportRedistributionRequest {
                    data_size,
                    grant_id,
                    normalized_cids: self.cids.clone(),
                },
            )
            .await?;

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.metadata_id.clone())
    }
}
