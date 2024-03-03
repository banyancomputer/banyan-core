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
pub struct ReportAllUserStorageTask;

#[async_trait]
impl TaskLike for ReportAllUserStorageTask {
    const TASK_NAME: &'static str = "report_all_users_storage_task";

    type Error = StorageReporterTaskError;
    type Context = StorageReporterTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.database().acquire().await?;
        let slot_end = round_to_previous_hour(OffsetDateTime::now_utc())?;

        let users = sqlx::query!("SELECT DISTINCT user_id FROM main.buckets")
            .fetch_all(&mut *db_conn)
            .await?;

        for user in users {
            let storage_hosts = sqlx::query!(
                "SELECT DISTINCT storage_host_id FROM storage_hosts_metadatas_storage_grants shms
                INNER JOIN metadata AS m ON m.id = shms.metadata_id
                INNER JOIN main.buckets b ON m.bucket_id = b.id
                WHERE b.user_id = $1;",
                user.user_id,
            )
            .fetch_all(&mut *db_conn)
            .await?;

            for storage_host in storage_hosts {
                let user_storage_report = StorageHost::user_report(
                    &mut db_conn,
                    &storage_host.storage_host_id,
                    &user.user_id,
                )
                .await?;

                let hot_storage_bytes = user_storage_report.current_consumption()
                    + user_storage_report.current_metadata_size();
                if hot_storage_bytes == 0 {
                    continue;
                }
                match MetricsStorage::find_by_slot_user_and_storage_host(
                    &mut db_conn,
                    slot_end,
                    user.user_id.clone(),
                    storage_host.storage_host_id.clone(),
                )
                .await
                {
                    Ok(Some(existing_metrics)) => {
                        let hot_storage_bytes =
                            existing_metrics.hot_storage_bytes.max(hot_storage_bytes);
                        existing_metrics
                            .update(&mut db_conn, hot_storage_bytes, 0)
                            .await?;
                    }
                    Ok(None) => {
                        let new_metrics = MetricsStorage {
                            user_id: user.user_id.clone(),
                            hot_storage_bytes,
                            archival_storage_bytes: 0,
                            storage_host_id: storage_host.storage_host_id.clone(),
                            slot: slot_end,
                        };
                        new_metrics.save(&mut db_conn).await?;
                    }
                    Err(e) => return Err(StorageReporterTaskError::Sqlx(e)),
                }
            }
        }

        Ok(())
    }
}
