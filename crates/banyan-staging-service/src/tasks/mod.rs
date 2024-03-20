mod prune_blocks;
mod redistribute_data;
mod report_bandwidth_metrics;
mod report_health;
mod report_upload;
mod upload_blocks;

use banyan_task::{QueueConfig, SqliteTaskStore, TaskLike, TaskLikeExt, TaskStore, WorkerPool};
pub use prune_blocks::PruneBlocksTask;
pub use redistribute_data::RedistributeDataTask;
pub use report_health::ReportHealthTask;
pub use report_upload::ReportUploadTask;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;
use crate::database::DatabaseConnection;
use crate::tasks::report_bandwidth_metrics::ReportBandwidthMetricsTask;
pub use crate::tasks::upload_blocks::UploadBlocksTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());
    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<ReportUploadTask>()
        .register_task_type::<PruneBlocksTask>()
        .register_task_type::<RedistributeDataTask>()
        .register_task_type::<UploadBlocksTask>()
        .register_recurring_task_type::<ReportHealthTask>()
        .register_recurring_task_type::<ReportBandwidthMetricsTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "prune blocks worker startup failed")
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
