#![allow(dead_code)]

use std::time::Duration;

mod models;
pub mod panic_safe_future;
mod queue_config;
mod stores;
mod task_instance_builder;
mod task_like;
mod task_store;
mod worker;
mod worker_pool;

pub use models::current_task::{CurrentTask, CurrentTaskError};
pub use models::task::{Task, TaskExecError};
pub use models::task_state::TaskState;
pub use queue_config::QueueConfig;
pub use stores::SqliteTaskStore;
pub use task_instance_builder::TaskInstanceBuilder;
pub use task_like::{TaskLike, TaskLikeExt};
pub use task_store::{TaskStore, TaskStoreError, TaskStoreMetrics};
pub use worker::{Worker, WorkerError};
pub use worker_pool::{ExecuteTaskFn, StateFn, WorkerPool, WorkerPoolError};

pub mod tests {
    pub use task_like::tests::TestTask;
    pub use task_store::tests::default_task_store_metrics;

    use super::{task_like, task_store};
}

pub const MAXIMUM_CHECK_DELAY: Duration = Duration::from_millis(250);

pub const TASK_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

pub const WORKER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
