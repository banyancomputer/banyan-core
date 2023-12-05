#![allow(dead_code)]

use std::time::Duration;

mod current_task;
pub mod panic_safe_future;
mod queue_config;
mod stores;
mod task;
mod task_instance_builder;
mod task_like;
mod task_state;
mod task_store;
mod worker;
mod worker_pool;

pub use current_task::{CurrentTask, CurrentTaskError};
pub use queue_config::QueueConfig;
pub use stores::{SqliteTaskStore, SqliteTaskStoreMetrics};
pub use task::{Task, TaskExecError};
pub use task_instance_builder::TaskInstanceBuilder;
pub use task_like::{TaskLike, TaskLikeExt};
pub use task_state::TaskState;
pub use task_store::{TaskStore, TaskStoreError};
pub use worker::{Worker, WorkerError};
pub use worker_pool::{ExecuteTaskFn, StateFn, WorkerPool, WorkerPoolError};

pub mod tests {
    use super::current_task;
    pub use current_task::tests::{default_current_task, increment_current_task_attempt_count};
}

pub const MAXIMUM_CHECK_DELAY: Duration = Duration::from_millis(250);

pub const TASK_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

pub const WORKER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
