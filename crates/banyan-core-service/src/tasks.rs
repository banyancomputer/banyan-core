mod email;
mod prune_blocks;

#[allow(unused_imports)]
pub use email::{
    EmailTaskContext, EmailTaskError, GaReleaseEmailTask, PaymentFailedEmailTask,
    ProductInvoiceEmailTask, ReachingStorageLimitEmailTask, ScheduledMaintenanceEmailTask,
};
pub use prune_blocks::{PruneBlock, PruneBlocksTask};

use tokio::sync::watch;
use tokio::task::JoinHandle;

use banyan_task::{QueueConfig, SqliteTaskStore, WorkerPool};

use crate::app::AppState;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());

    WorkerPool::new(task_store.clone(), move || state.clone())
        // TODO: until we fix concurrency issues, only run one worker
        .configure_queue(QueueConfig::new("default").with_worker_count(1))
        .register_task_type::<PruneBlocksTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "background worker startup failed")
}
