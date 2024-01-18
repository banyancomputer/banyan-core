use time::OffsetDateTime;

use crate::{Task, TaskLike, TaskState, TaskStoreError};

pub struct TaskInstanceBuilder {
    pub task_name: String,
    pub queue_name: String,
    pub maximum_attempts: i64,

    pub payload: Vec<u8>,
    pub current_attempt: i64,
    pub state: TaskState,

    pub original_task_id: Option<String>,
    pub unique_key: Option<String>,

    pub scheduled_to_run_at: OffsetDateTime,
}

impl TaskInstanceBuilder {
    pub async fn for_task<T: TaskLike>(instance: T) -> Result<Self, TaskStoreError> {
        let unique_key = instance.unique_key();
        let payload = serde_json::to_vec(&instance).map_err(TaskStoreError::EncodeFailed)?;

        Ok(Self {
            task_name: T::TASK_NAME.to_string(),
            queue_name: T::QUEUE_NAME.to_string(),
            maximum_attempts: T::MAX_ATTEMPTS,

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

    pub fn reset_task(mut self) -> Self {
        self.current_attempt = 0;
        self.state = TaskState::New;
        self.original_task_id = None;
        self
    }

    pub fn run_at(mut self, time: OffsetDateTime) -> Self {
        self.scheduled_to_run_at = time;
        self
    }
}
