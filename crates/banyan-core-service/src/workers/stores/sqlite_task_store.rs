use async_trait::async_trait;
use sqlx::{Sqlite, SqliteConnection, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::workers::{TASK_EXECUTION_TIMEOUT, Task, TaskId, TaskLike, TaskState, TaskStore, TaskStoreError};

#[derive(Clone, Default)]
pub struct SqliteTaskStore;

impl SqliteTaskStore {
    async fn is_key_present(pool: &SqlitePool, key: &str) -> Result<bool, TaskStoreError> {
        let query_res = sqlx::query_scalar!("SELECT 1 FROM background_tasks WHERE unique_key = $1;", key)
            .fetch_optional(&*pool)
            .await
            .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        Ok(query_res.is_some())
    }
}

#[async_trait]
impl TaskStore for SqliteTaskStore {
    type Connection = SqlitePool;

    async fn enqueue<T: TaskLike>(
        pool: &mut Self::Connection,
        task: T,
    ) -> Result<Option<TaskId>, TaskStoreError> {
        let unique_key = task.unique_key().await;

        if let Some(ukey) = &unique_key {
            // right now if we encounter a unique key that is already present in the DB we simply
            // don't queue the new instance of that task, the old one will have a bit of priority
            // due to its age.
            if SqliteTaskStore::is_key_present(pool, ukey).await? {
                return Ok(None);
            }
        }

        let payload = serde_json::to_string(&task)
            .map_err(TaskStoreError::EncodeFailed)?;

        let task_name = T::TASK_NAME.to_string();
        let queue_name = T::QUEUE_NAME.to_string();

        let background_task_id: String = sqlx::query_scalar!(
                r#"INSERT INTO background_tasks (task_name, unique_key, queue_name, payload, maximum_attempts)
                    VALUES ($1, $2, $3, $4, $5)
                    RETURNING id;"#,
                task_name,
                unique_key,
                queue_name,
                payload,
                T::MAX_RETRIES,
            )
            .fetch_one(&*pool)
            .await
            .map_err(|err| TaskStoreError::ConnectionFailure(err.to_string()))?;

        Ok(Some(TaskId::from(Uuid::parse_str(background_task_id.as_str()).expect("valid uuid"))))
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
