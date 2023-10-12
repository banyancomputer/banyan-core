#![allow(dead_code)]

use async_trait::async_trait;

use crate::workers::{Task, TaskExecError, TaskLike, TaskState};

#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    type Connection: Send;

    async fn cancel(&self, id: String) -> Result<(), TaskStoreError> {
        self.update_state(id, TaskState::Cancelled).await
    }

    async fn completed(&self, id: String) -> Result<(), TaskStoreError> {
        self.update_state(id, TaskState::Complete).await
    }

    async fn enqueue<T: TaskLike>(
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
