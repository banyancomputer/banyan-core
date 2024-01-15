use async_trait::async_trait;
use serde::Serialize;

use crate::{Task, TaskExecError, TaskLike, TaskState};

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct TaskStoreMetrics {
    pub(crate) total: i32,
    pub(crate) new: i32,
    pub(crate) in_progress: i32,
    pub(crate) panicked: i32,
    pub(crate) retried: i32,
    pub(crate) cancelled: i32,
    pub(crate) errored: i32,
    pub(crate) completed: i32,
    pub(crate) timed_out: i32,
    pub(crate) dead: i32,
    pub(crate) scheduled: i32,
    pub(crate) scheduled_future: i32,
}

#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    type Pool: Send;
    type Connection: Send;

    async fn cancel(&self, id: String) -> Result<(), TaskStoreError> {
        self.update_state(id, TaskState::Cancelled).await
    }

    async fn completed(&self, id: String) -> Result<(), TaskStoreError> {
        self.update_state(id, TaskState::Complete).await
    }

    async fn enqueue<T: TaskLike>(
        pool: &mut Self::Pool,
        task: T,
    ) -> Result<Option<String>, TaskStoreError>
    where
        Self: Sized;

    async fn enqueue_with_connection<T: TaskLike>(
        conn: &mut Self::Connection,
        task: T,
    ) -> Result<Option<String>, TaskStoreError>
    where
        Self: Sized;

    async fn errored(
        &self,
        id: String,
        error: TaskExecError,
    ) -> Result<Option<String>, TaskStoreError> {
        match error {
            TaskExecError::DeserializationFailed(_) | TaskExecError::Panicked(_) => {
                self.update_state(id, TaskState::Dead).await?;
                Ok(None)
            }
            TaskExecError::ExecutionFailed(_) => {
                self.update_state(id.clone(), TaskState::Error).await?;
                self.retry(id).await
            }
        }
    }

    async fn next(
        &self,
        queue_name: &str,
        task_names: &[&str],
    ) -> Result<Option<Task>, TaskStoreError>;

    async fn retry(&self, id: String) -> Result<Option<String>, TaskStoreError>;

    async fn update_state(&self, id: String, state: TaskState) -> Result<(), TaskStoreError>;

    async fn schedule_next<T: TaskLike>(&self, task: T) -> Result<Option<String>, TaskStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum TaskStoreError {
    #[error("the underlying connection experienced an issue: {0}")]
    ConnectionFailure(String),

    #[error("failed to encode task as JSON: {0}")]
    EncodeFailed(serde_json::Error),

    #[error("a task can't transition between {0:?} and {1:?}")]
    InvalidStateTransition(TaskState, TaskState),

    #[error("unable to retry task from invalid state '{0:?}'")]
    NotRetryable(TaskState),

    #[error("unable to find task with ID {0}")]
    UnknownTask(String),
}

impl From<sqlx::Error> for TaskStoreError {
    fn from(value: sqlx::Error) -> Self {
        TaskStoreError::ConnectionFailure(value.to_string())
    }
}

pub mod tests {
    use super::TaskStoreMetrics;

    pub fn default_task_store_metrics() -> TaskStoreMetrics {
        TaskStoreMetrics {
            total: 0,
            new: 0,
            in_progress: 0,
            panicked: 0,
            retried: 0,
            cancelled: 0,
            errored: 0,
            completed: 0,
            timed_out: 0,
            dead: 0,
            scheduled: 0,
            scheduled_future: 0,
        }
    }
}
