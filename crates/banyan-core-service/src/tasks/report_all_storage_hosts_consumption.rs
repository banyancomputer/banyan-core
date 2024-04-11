use async_trait::async_trait;
use banyan_task::{CurrentTask, RecurringTask, RecurringTaskError, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::tasks::report_storage_host_consumption::save_storage_host_consumption;
use crate::utils::time::round_to_next_hour;

pub type StorageReporterTaskContext = AppState;

#[derive(Debug, thiserror::Error)]
pub enum StorageReporterTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("could not calculate end slot: {0}")]
    EndSlotParsingError(#[from] ComponentRange),
}

#[derive(Deserialize, Serialize, Default)]
pub struct ReportAllStorageHostsConsumptionTask {}

#[async_trait]
impl TaskLike for ReportAllStorageHostsConsumptionTask {
    const TASK_NAME: &'static str = "report_all_storage_hosts_consumption_task";

    type Error = StorageReporterTaskError;
    type Context = StorageReporterTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut conn = ctx.database().acquire().await?;
        let slot_end = round_to_next_hour(OffsetDateTime::now_utc())?;

        let storage_hosts = sqlx::query_scalar!("SELECT DISTINCT id FROM storage_hosts")
            .fetch_all(&mut *conn)
            .await?;

        for host_id in storage_hosts {
            save_storage_host_consumption(&mut conn, slot_end, &host_id).await?;
        }

        Ok(())
    }
}

impl RecurringTask for ReportAllStorageHostsConsumptionTask {
    fn next_schedule(&self) -> Result<Option<OffsetDateTime>, RecurringTaskError> {
        OffsetDateTime::now_utc()
            .checked_add(time::Duration::hours(1))
            .ok_or(RecurringTaskError::DateTimeAddition)
            .map(Some)
    }
}
