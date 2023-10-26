mod prune_blocks;

pub use prune_blocks::{PruneBlock, PruneBlocksTask, PruneBlocksTaskContext, PruneBlocksTaskError};

use sqlx::SqlitePool;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use banyan_task::{QueueConfig, SqliteTaskStore, WorkerPool};

pub async fn start_prune_blocks_workers(
    pool: SqlitePool,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, &'static str> {
    let task_store = SqliteTaskStore::new(pool.clone());

    WorkerPool::new(task_store.clone(), move || pool.clone())
        .configure_queue(QueueConfig::new("prune_blocks").with_worker_count(5))
        .register_task_type::<PruneBlocksTask>()
        .start(async move {
            let _ = shutdown_rx.changed().await;
        })
        .await
        .map_err(|_| "prune blocks worker startup failed")
}
