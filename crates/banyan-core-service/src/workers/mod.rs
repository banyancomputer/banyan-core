use std::time::Duration;

mod current_task;
pub mod panic_safe_future;
mod queue_config;
mod stores;
mod task;
mod task_like;
mod task_state;
mod task_store;
pub mod tasks;
mod worker;
mod worker_pool;

pub use current_task::CurrentTask;
pub use queue_config::QueueConfig;
//pub use stores::{MemoryTaskStore, SqliteTaskStore};
pub use stores::SqliteTaskStore;
pub use task_like::{TaskLike, TaskLikeExt};
pub use task_state::TaskState;
pub use task_store::{TaskStore, TaskStoreError};
pub use task::{Task, TaskExecError};
pub use worker::{Worker, WorkerError};
pub use worker_pool::{ExecuteTaskFn, StateFn, WorkerPool, WorkerPoolError};

pub const MAXIMUM_CHECK_DELAY: Duration = Duration::from_millis(250);

pub const TASK_EXECUTION_TIMEOUT: Duration = Duration::from_secs(30);

pub const WORKER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
