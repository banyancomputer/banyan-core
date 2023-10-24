use time::OffsetDateTime;

use crate::panic_safe_future::CaughtPanic;
use crate::TaskState;

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub original_task_id: Option<String>,

    pub task_name: String,
    pub queue_name: String,

    pub unique_key: Option<String>,
    pub state: TaskState,

    pub current_attempt: i64,
    pub maximum_attempts: i64,

    // will need a live-cancel signal and likely a custom Future impl to ensure its used for proper
    // timeout handling
    pub payload: serde_json::Value,
    pub error: Option<String>,

    pub scheduled_at: OffsetDateTime,
    pub scheduled_to_run_at: OffsetDateTime,

    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskExecError {
    #[error("task deserialization failed: {0}")]
    DeserializationFailed(#[from] serde_json::Error),

    #[error("task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("task panicked: {0}")]
    Panicked(#[from] CaughtPanic),
}
