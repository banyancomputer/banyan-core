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
pub use task_like::{RecurringTask, TaskLike, TaskLikeExt};
pub use task_store::{TaskStore, TaskStoreError, TaskStoreMetrics};
pub use worker::{Worker, WorkerError};
pub use worker_pool::{ExecuteTaskFn, NextScheduleFn, StateFn, WorkerPool, WorkerPoolError};

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    pub use task_like::tests::TestTask;

    use super::task_like;
    pub use crate::models::current_task;
    pub use crate::models::current_task::tests::increment_current_task_attempt_count;
}

pub const MAXIMUM_CHECK_DELAY: Duration = Duration::from_millis(250);

pub const TASK_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

pub const WORKER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
