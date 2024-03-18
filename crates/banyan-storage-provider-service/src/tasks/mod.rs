mod prune_blocks;
mod report_bandwidth_metrics;
mod report_health;
mod report_upload;

use banyan_task::{QueueConfig, SqliteTaskStore, WorkerPool};
pub use prune_blocks::PruneBlocksTask;
pub use report_health::ReportHealthTask;
pub use report_upload::ReportUploadTask;
use sqlx::{Connection, Sqlite, SqliteConnection, Transaction};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;
use crate::tasks::report_bandwidth_metrics::ReportBandwidthMetricsTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, ()> {
    let task_store = SqliteTaskStore::new(state.database());
    let database = state.database();
    let mut connection = database.acquire().await.unwrap();

    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<ReportUploadTask>()
        .register_task_type::<PruneBlocksTask>()
        .register_recurring_task_type::<ReportHealthTask>()
        .register_recurring_task_type::<ReportBandwidthMetricsTask>()
        .start(&mut *connection, async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| ())
}
