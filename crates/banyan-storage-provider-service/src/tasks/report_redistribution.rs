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
    #[error("invalid cid")]
    InvalidInternalCid,
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    CoreServiceError(#[from] CoreServiceError),
}

#[derive(Deserialize, Serialize)]
pub struct ReportRedistributionTask {
    grant_id: Uuid,
    replication: bool,
    metadata_id: String,
    cids: Vec<String>,
    data_size: i64,
}

impl ReportRedistributionTask {
    pub fn new(
        grant_id: Uuid,
        metadata_id: &str,
        cids: &[String],
        data_size: i64,
        replication: bool,
    ) -> Self {
        Self {
            grant_id,
            metadata_id: String::from(metadata_id),
            cids: cids.to_vec(),
            data_size,
            replication,
        }
    }
}

#[async_trait]
impl TaskLike for ReportRedistributionTask {
    const TASK_NAME: &'static str = "report_redistribution_task";

    type Error = ReportRedistributionTaskError;
    type Context = ReportRedistributionTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
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
                    replication: self.replication,
                    data_size: self.data_size,
                    grant_id: self.grant_id.to_string(),
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
