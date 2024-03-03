use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::database::models::{MetricsStorage, StorageHost};
use crate::utils::time::round_to_previous_hour;

pub type StorageReporterTaskContext = AppState;

#[derive(Debug, thiserror::Error)]
pub enum StorageReporterTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("could not calculate end slot: {0}")]
    EndSlotParsingError(#[from] ComponentRange),
}

#[derive(Deserialize, Serialize)]
pub struct ReportUserStorage {
    user_id: String,
    storage_host_id: String,
}

impl ReportUserStorage {
    pub fn new(user_id: String, storage_host_id: String) -> Self {
        Self {
            user_id,
            storage_host_id,
        }
    }
}

#[async_trait]
impl TaskLike for ReportUserStorage {
    const TASK_NAME: &'static str = "report_user_storage_task";

    type Error = StorageReporterTaskError;
    type Context = StorageReporterTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.database().acquire().await?;
        let slot_end = round_to_previous_hour(OffsetDateTime::now_utc())?;

        let user_storage_report =
            StorageHost::user_report(&mut db_conn, &self.storage_host_id, &self.user_id).await?;

        let hot_storage_bytes =
            user_storage_report.current_consumption() + user_storage_report.current_metadata_size();
        match MetricsStorage::find_by_slot_user_and_storage_host(
            &mut db_conn,
            slot_end,
            self.user_id.clone(),
            self.storage_host_id.clone(),
        )
        .await
        {
            Ok(Some(existing_metrics)) => {
                let updated_hot_storage_bytes =
                    std::cmp::max(existing_metrics.hot_storage_bytes, hot_storage_bytes);
                existing_metrics
                    .update(
                        &mut db_conn,
                        updated_hot_storage_bytes,
                        existing_metrics.archival_storage_bytes,
                    )
                    .await?;
            }
            Ok(None) => {
                MetricsStorage {
                    user_id: self.user_id.clone(),
                    hot_storage_bytes,
                    archival_storage_bytes: 0,
                    storage_host_id: self.storage_host_id.clone(),
                    slot: slot_end,
                }
                .save(&mut db_conn)
                .await?;
            }
            Err(e) => return Err(StorageReporterTaskError::Sqlx(e)),
        }

        Ok(())
    }
}
