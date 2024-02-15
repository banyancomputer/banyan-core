use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;

pub type ReportDistributionCompleteTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportDistributionCompleteTaskError {
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
pub struct ReportDistributionCompleteTask {
    metadata_id: String,
    new_upload_id: String,
}

impl ReportDistributionCompleteTask {
    pub fn new(metadata_id: String, new_upload_id: String) -> Self {
        Self {
            metadata_id,
            new_upload_id,
        }
    }
}

#[async_trait]
impl TaskLike for ReportDistributionCompleteTask {
    const TASK_NAME: &'static str = "redistribute_data_task";

    type Error = ReportDistributionCompleteTaskError;
    type Context = ReportDistributionCompleteTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        Ok(())
    }
}
