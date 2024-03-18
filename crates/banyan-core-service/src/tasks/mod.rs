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

use banyan_task::{QueueConfig, SqliteTaskStore, TaskLike, TaskLikeExt, TaskStore, WorkerPool};
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
use crate::database::DatabaseConnection;
use crate::tasks::redistribute_staging_data::RedistributeStagingDataTask;
use crate::tasks::report_all_storage_hosts_consumption::ReportAllStorageHostsConsumptionTask;
use crate::tasks::report_all_users_consumption::ReportAllUsersConsumptionTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());

    let mut conn = state.database().acquire().await.unwrap();

    enqueue_task_if_none_in_progress::<ReportAllUsersConsumptionTask>(&task_store, &mut conn).await;
    enqueue_task_if_none_in_progress::<ReportAllUsersConsumptionTask>(&task_store, &mut conn).await;

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
    conn: &mut DatabaseConnection,
) {
    if task_store
        .get_living_task(T::TASK_NAME)
        .await
        .expect("get task")
        .is_some()
    {
        return;
    }

    T::default()
        .enqueue::<SqliteTaskStore>(conn)
        .await
        .expect("enqueue task");
}
