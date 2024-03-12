use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::database::models::{StorageHost, StorageHostTotalConsumption};
use crate::database::DatabaseConnection;
use crate::utils::time::round_to_next_hour;

pub type StorageReporterTaskContext = AppState;

#[derive(Debug, thiserror::Error)]
pub enum StorageReporterTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("could not calculate end slot: {0}")]
    EndSlotParsingError(#[from] ComponentRange),
}

#[derive(Deserialize, Serialize)]
pub struct ReportStorageHostConsumptionTask {
    storage_host_id: String,
}

impl ReportStorageHostConsumptionTask {
    pub fn new(storage_host_id: String) -> Self {
        Self { storage_host_id }
    }
}

#[async_trait]
impl TaskLike for ReportStorageHostConsumptionTask {
    const TASK_NAME: &'static str = "report_storage_host_consumption_task";

    type Error = StorageReporterTaskError;
    type Context = StorageReporterTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.database().acquire().await?;
        // round up as it makes more sense when looking backwards
        let slot_end = round_to_next_hour(OffsetDateTime::now_utc())?;

        save_storage_host_consumption(&mut db_conn, slot_end, &self.storage_host_id).await?;

        Ok(())
    }
}

pub async fn save_storage_host_consumption(
    conn: &mut DatabaseConnection,
    slot_end: OffsetDateTime,
    storage_host_id: &str,
) -> Result<(), sqlx::Error> {
    let storage_bytes = StorageHost::total_consumption(conn, storage_host_id).await?;
    match StorageHostTotalConsumption::find_by_slot_and_host(conn, slot_end, storage_host_id).await
    {
        Ok(Some(existing_metrics)) => {
            let updated_storage_bytes =
                std::cmp::max(existing_metrics.storage_bytes, storage_bytes);
            existing_metrics.update(conn, updated_storage_bytes).await?;
        }
        Ok(None) => {
            StorageHostTotalConsumption {
                storage_host_id: storage_host_id.to_string(),
                storage_bytes,
                slot: slot_end,
            }
            .save(conn)
            .await?;
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
