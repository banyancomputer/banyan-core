use sqlx::SqliteConnection;
use time::OffsetDateTime;

use crate::workers::{SqliteTaskStore, Task, TaskLike, TaskState, TaskStoreError};

pub struct TaskInstanceBuilder {
    task_name: String,
    queue_name: String,
    maximum_attempts: i64,

    payload: serde_json::Value,
    current_attempt: i64,
    state: TaskState,

    original_task_id: Option<String>,
    unique_key: Option<String>,

    scheduled_to_run_at: OffsetDateTime,
}

impl TaskInstanceBuilder {
    pub async fn create(
        self,
        conn: &mut SqliteConnection,
    ) -> Result<Option<String>, TaskStoreError> {
        if let Some(ukey) = &self.unique_key {
            // right now if we encounter a unique key that is already present in the DB we simply
            // don't queue the new instance of that task, the old one will have a bit of priority
            // due to its age.
            if SqliteTaskStore::is_key_present(conn, ukey, &self.task_name).await? {
                return Ok(None);
            }
        }

        let payload = self.payload.to_string();
        let background_task_id: String = sqlx::query_scalar!(
            r#"INSERT INTO background_tasks (
                           task_name, queue_name, unique_key, payload,
                           current_attempt, maximum_attempts, state,
                           original_task_id, scheduled_to_run_at
                       )
                       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                       RETURNING id;"#,
            self.task_name,
            self.queue_name,
            self.unique_key,
            payload,
            self.current_attempt,
            self.maximum_attempts,
            self.state,
            self.original_task_id,
            self.scheduled_to_run_at,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(Some(background_task_id))
    }

    pub async fn for_task<T: TaskLike>(instance: T) -> Result<Self, TaskStoreError> {
        let unique_key = instance.unique_key().await;
        let payload = serde_json::to_value(&instance).map_err(TaskStoreError::EncodeFailed)?;

        Ok(Self {
            task_name: T::TASK_NAME.to_string(),
            queue_name: T::QUEUE_NAME.to_string(),
            maximum_attempts: T::MAX_RETRIES,

            payload,
            current_attempt: 0,
            state: TaskState::New,

            original_task_id: None,
            unique_key,

            scheduled_to_run_at: OffsetDateTime::now_utc(),
        })
    }

    pub async fn from_task_instance(task: Task) -> Self {
        Self {
            task_name: task.task_name,
            queue_name: task.queue_name,
            maximum_attempts: task.maximum_attempts,

            payload: task.payload,
            current_attempt: task.current_attempt + 1,
            state: TaskState::Retry,

            original_task_id: Some(task.original_task_id.unwrap_or(task.id)),
            unique_key: task.unique_key,

            scheduled_to_run_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn run_at(mut self, time: OffsetDateTime) -> Self {
        self.scheduled_to_run_at = time;
        self
    }
}
