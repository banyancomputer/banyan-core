use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::app::AppState;

#[derive(Debug, thiserror::Error)]
pub enum CheckDistributionCompleteTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Deserialize, Serialize)]
pub struct CheckDistributionCompleteTask {}

#[async_trait]
impl TaskLike for CheckDistributionCompleteTask {
    const TASK_NAME: &'static str = "check_distribution_complete_task";

    type Error = CheckDistributionCompleteTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, _ctx: Self::Context) -> Result<(), Self::Error> {
        Ok(())
    }

    fn next_time(&self) -> Option<OffsetDateTime> {
        Some(OffsetDateTime::now_utc() + time::Duration::days(1))
    }
}
