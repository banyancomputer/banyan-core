mod prune_blocks;
mod report_bandwidth_metrics;
mod report_health;
mod report_redistribution;
mod report_upload;

use banyan_task::{QueueConfig, SqliteTaskStore, TaskLike, TaskLikeExt, TaskState, WorkerPool};
pub use prune_blocks::PruneBlocksTask;
pub use report_health::ReportHealthTask;
pub use report_redistribution::ReportRedistributionTask;
pub use report_upload::ReportUploadTask;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;
use crate::database::Database;
use crate::tasks::report_bandwidth_metrics::ReportBandwidthMetricsTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());

    enqueue_task_if_none_in_progress::<ReportBandwidthMetricsTask>(
        &task_store,
        &mut state.database(),
    )
    .await;
    enqueue_task_if_none_in_progress::<ReportHealthTask>(&task_store, &mut state.database()).await;

    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<ReportUploadTask>()
        .register_task_type::<PruneBlocksTask>()
        .register_task_type::<ReportHealthTask>()
        .register_task_type::<ReportBandwidthMetricsTask>()
        .register_task_type::<ReportRedistributionTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "prune blocks worker startup failed")
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
