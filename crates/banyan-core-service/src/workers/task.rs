use chrono::NaiveDateTime;

use crate::workers::{TaskState};
use crate::workers::panic_safe_future::CaughtPanic;

#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct Task {
    pub id: String,

    pub next_id: Option<String>,
    pub previous_id: Option<String>,

    pub task_name: String,
    pub queue_name: String,

    pub unique_key: Option<String>,
    pub state: TaskState,

    pub current_attempt: i32,
    pub maximum_attempts: i32,

    // will need a live-cancel signal and likely a custom Future impl to ensure its used for proper
    // timeout handling

    pub payload: serde_json::Value,
    pub error: Option<String>,

    pub scheduled_at: NaiveDateTime,
    pub scheduled_to_run_at: NaiveDateTime,

    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
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
