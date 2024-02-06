use banyan_task::{QueueConfig, SqliteTaskStore, WorkerPool};
pub use prune_blocks::PruneBlocksTask;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;
use crate::tasks::complete_upload_blocks::CompleteUploadBlocksTask;
use crate::tasks::redistribute_data::RedistributeDataTask;
pub use crate::tasks::report_upload::ReportUploadTask;
use crate::tasks::upload_blocks::UploadBlocksTask;

mod complete_upload_blocks;
mod prune_blocks;
mod redistribute_data;
mod report_upload;
mod upload_blocks;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());

    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<ReportUploadTask>()
        .register_task_type::<RedistributeDataTask>()
        .register_task_type::<UploadBlocksTask>()
        .register_task_type::<CompleteUploadBlocksTask>()
        .register_task_type::<PruneBlocksTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "prune blocks worker startup failed")
}
