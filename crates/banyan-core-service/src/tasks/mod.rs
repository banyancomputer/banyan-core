mod create_deals;
mod delete_staging_data;
mod email;
mod host_capacity;
mod prune_blocks;
mod redistribute_staging_data;
mod replicate_data;
mod report_all_storage_hosts_consumption;
mod report_all_users_consumption;
mod report_storage_host_consumption;
mod report_user_consumption;

use banyan_task::{QueueConfig, SqliteTaskStore, WorkerPool};
pub use create_deals::{CreateDealsTask, BLOCK_SIZE};
pub use delete_staging_data::DeleteStagingDataTask;
#[allow(unused_imports)]
pub use email::{
    EmailTaskContext, EmailTaskError, GaReleaseEmailTask, PaymentFailedEmailTask,
    ProductInvoiceEmailTask, ReachingStorageLimitEmailTask, ScheduledMaintenanceEmailTask,
};
pub use host_capacity::HostCapacityTask;
pub use prune_blocks::PruneBlocksTask;
pub use report_storage_host_consumption::ReportStorageHostConsumptionTask;
pub use report_user_consumption::ReportUserConsumptionTask;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;
use crate::tasks::redistribute_staging_data::RedistributeStagingDataTask;
use crate::tasks::replicate_data::ReplicateDataTask;
use crate::tasks::report_all_storage_hosts_consumption::ReportAllStorageHostsConsumptionTask;
use crate::tasks::report_all_users_consumption::ReportAllUsersConsumptionTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());
    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<PruneBlocksTask>()
        .register_task_type::<CreateDealsTask>()
        .register_task_type::<ReportUserConsumptionTask>()
        .register_task_type::<ReportAllUsersConsumptionTask>()
        .register_task_type::<ReportStorageHostConsumptionTask>()
        .register_task_type::<DeleteStagingDataTask>()
        .register_task_type::<HostCapacityTask>()
        .register_recurring_task_type::<ReplicateDataTask>()
        .register_recurring_task_type::<RedistributeStagingDataTask>()
        .register_recurring_task_type::<ReportAllStorageHostsConsumptionTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "background worker startup failed")
}
