mod create_deals;
mod delete_staging_data;
mod email;
mod host_capacity;
mod prune_blocks;
mod redistribute_staging_data;
mod report_all_storage_hosts_consumption;
mod report_all_users_consumption;
mod report_storage_host_consumption;
mod report_user_consumption;

use banyan_task::{QueueConfig, SqliteTaskStore, TaskLike, TaskLikeExt, TaskState, WorkerPool};
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
use crate::database::Database;
use crate::tasks::redistribute_staging_data::RedistributeStagingDataTask;
use crate::tasks::report_all_storage_hosts_consumption::ReportAllStorageHostsConsumptionTask;
use crate::tasks::report_all_users_consumption::ReportAllUsersConsumptionTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());

    enqueue_task_if_none_in_progress::<ReportAllUsersConsumptionTask>(
        &task_store,
        &mut state.database(),
    )
    .await;
    enqueue_task_if_none_in_progress::<ReportAllUsersConsumptionTask>(
        &task_store,
        &mut state.database(),
    )
    .await;
    // TODO: uncomment after testing with small uploads on production
    enqueue_task_if_none_in_progress::<RedistributeStagingDataTask>(
        &task_store,
        &mut state.database(),
    )
    .await;

    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<PruneBlocksTask>()
        .register_task_type::<CreateDealsTask>()
        .register_task_type::<RedistributeStagingDataTask>()
        .register_task_type::<ReportUserConsumptionTask>()
        .register_task_type::<ReportAllUsersConsumptionTask>()
        .register_task_type::<ReportStorageHostConsumptionTask>()
        .register_task_type::<ReportAllStorageHostsConsumptionTask>()
        .register_task_type::<DeleteStagingDataTask>()
        .register_task_type::<HostCapacityTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "background worker startup failed")
}
async fn enqueue_task_if_none_in_progress<T: TaskLikeExt + TaskLike + Default>(
    task_store: &SqliteTaskStore,
    db: &mut Database,
) {
    if task_store
        .task_in_state::<T>(vec![TaskState::New, TaskState::Retry])
        .await
        .expect("get task")
        .is_some()
    {
        return;
    }

    T::default()
        .enqueue::<SqliteTaskStore>(db)
        .await
        .expect("enqueue task");
}
