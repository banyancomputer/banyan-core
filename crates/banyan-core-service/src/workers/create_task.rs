use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::workers::TaskLike;

#[derive(Deserialize, Serialize)]
pub struct CreateTask {
    name: String,
    queue_name: String,

    payload: serde_json::Value,
    maximum_attempts: usize,

    scheduled_to_run_at: OffsetDateTime,
}

impl CreateTask {
    pub fn new<T: TaskLike>(task: T, run_at: OffsetDateTime) -> Self {
        Self {
            name: T::TASK_NAME.to_string(),
            queue_name: T::QUEUE_NAME.to_string(),

            payload: serde_json::to_value(&task).expect("valid encoding"),
            maximum_attempts: T::MAX_RETRIES,

            scheduled_to_run_at: run_at,
        }
    }
}
