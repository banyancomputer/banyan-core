use async_trait::async_trait;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::workers::{TASK_EXECUTION_TIMEOUT, CreateTask, Task, TaskId, TaskLike, TaskState, TaskStore, TaskStoreError};

#[derive(Clone, Default)]
pub struct SqliteTaskStore;

impl SqliteTaskStore {
    async fn is_key_present(conn: &SqlitePool, key: &str) -> Result<bool, TaskStoreError> {
        let query_res = sqlx::query_scalar!("SELECT 1 FROM background_tasks WHERE unique_key = $1;", key)
            .fetch_optional(conn)
            .await
            .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        Ok(query_res.is_some())
    }
}

#[async_trait]
impl TaskStore for SqliteTaskStore {
    type Connection = SqlitePool;

    async fn enqueue<T: TaskLike>(
        conn: &mut Self::Connection,
        task: T,
    ) -> Result<Option<TaskId>, TaskStoreError> {
        let unique_key = task.unique_key().await;

        if let Some(ukey) = &unique_key {
            // right now if we encounter a unique key that is already present in the DB we simply
            // don't queue the new instance of that task, the old one will have a bit of priority
            // due to its age.
            if SqliteTaskStore::is_key_present(conn, ukey).await? {
                return Ok(None);
            }
        }

        let new_task = CreateTask::new(task, OffsetDateTime::now_utc());

        todo!()
    }

    async fn next(&self, queue_name: &str, task_names: &[&str]) -> Result<Option<Task>, TaskStoreError> {
        todo!()
    }

    async fn retry(&self, id: TaskId) -> Result<Option<TaskId>, TaskStoreError> {
        todo!()
    }

    async fn update_state(&self, id: TaskId, new_state: TaskState) -> Result<(), TaskStoreError> {
        todo!()
    }
}
