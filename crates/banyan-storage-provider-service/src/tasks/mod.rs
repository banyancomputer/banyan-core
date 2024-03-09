mod prune_blocks;
mod report_bandwidth_metrics;
mod report_health;
mod report_upload;

use banyan_task::{QueueConfig, SqliteTaskStore, TaskLike, TaskLikeExt, TaskStore, WorkerPool};
pub use prune_blocks::PruneBlocksTask;
pub use report_health::ReportHealthTask;
pub use report_upload::ReportUploadTask;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;
use crate::tasks::report_bandwidth_metrics::ReportBandwidthMetricsTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, ()> {
    let task_store = SqliteTaskStore::new(state.database());
    /*
    // Enqueue a report bandwidth task if there is none in progress
    if task_store
        .get_living_task(ReportHealthTask::TASK_NAME)
        .await
        .expect("get report bandwidth metrics task")
        .is_none()
    {
        ReportBandwidthMetricsTask
            .enqueue::<SqliteTaskStore>(&mut state.database())
            .await
            .expect("enqueue report bandwidth metrics task");
    }

    // Enqueue a report health task if there is none in progress
    if task_store
        .get_living_task(ReportHealthTask::TASK_NAME)
        .await
        .expect("get report health task")
        .is_none()
    {
        ReportHealthTask
            .enqueue::<SqliteTaskStore>(&mut state.database())
            .await
            .expect("enqueue report health task");
    }
    */
    let state1 = state.clone();

    WorkerPool::new(
        task_store.clone(),
        move || state1.database(),
        move || state.clone(),
    )
    .configure_queue(QueueConfig::new("default").with_worker_count(5))
    .register_task_type::<ReportUploadTask>()
    .register_task_type::<PruneBlocksTask>()
    .register_recurring_task_type::<ReportHealthTask>()
    .await
    .map_err(|_| ())?
    .register_recurring_task_type::<ReportBandwidthMetricsTask>()
    .await
    .map_err(|_| ())?
    .start(async move {
        let _ = shutdown_rx.changed().await;
    })
    .await
    .map_err(|_| ())
}
