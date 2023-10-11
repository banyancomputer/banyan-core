use time::OffsetDateTime;

use crate::workers::{TaskId, TaskState};
use crate::workers::panic_safe_future::CaughtPanic;

#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub id: TaskId,

    next_id: Option<TaskId>,
    previous_id: Option<TaskId>,

    name: String,
    queue_name: String,

    unique_key: Option<String>,
    state: TaskState,

    current_attempt: usize,
    maximum_attempts: usize,

    // will need a live-cancel signal and likely a custom Future impl to ensure its used for proper
    // timeout handling

    payload: serde_json::Value,
    error: Option<String>,

    scheduled_at: OffsetDateTime,
    scheduled_to_run_at: OffsetDateTime,

    started_at: Option<OffsetDateTime>,
    finished_at: Option<OffsetDateTime>,
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
