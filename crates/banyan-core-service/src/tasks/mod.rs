mod create_deals;
mod email;
mod host_capacity;
mod prune_blocks;

use banyan_task::{QueueConfig, SqliteTaskStore, WorkerPool};
pub use create_deals::{CreateDealsTask, BLOCK_SIZE};
#[allow(unused_imports)]
pub use email::{
    EmailTaskContext, EmailTaskError, GaReleaseEmailTask, PaymentFailedEmailTask,
    ProductInvoiceEmailTask, ReachingStorageLimitEmailTask, ScheduledMaintenanceEmailTask,
};
pub use host_capacity::HostCapacityTask;
pub use prune_blocks::PruneBlocksTask;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(state.database());
    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<PruneBlocksTask>()
        .register_task_type::<CreateDealsTask>()
        .register_task_type::<HostCapacityTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "background worker startup failed")
}
