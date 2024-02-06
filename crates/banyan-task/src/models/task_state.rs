#[derive(Clone, Copy, Debug, Eq, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum TaskState {
    New,
    InProgress,
    Panicked,
    Retry,
    Cancelled,
    Error,
    Complete,
    TimedOut,
    Dead,
}

impl From<String> for TaskState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "new" => TaskState::New,
            "in_progress" => TaskState::InProgress,
            "panicked" => TaskState::Panicked,
            "retry" => TaskState::Retry,
            "cancelled" => TaskState::Cancelled,
            "error" => TaskState::Error,
            "complete" => TaskState::Complete,
            "timed_out" => TaskState::TimedOut,
            "dead" => TaskState::Dead,
            _ => panic!("unknown task state from string"),
        }
    }
}

impl From<TaskState> for String {
    fn from(value: TaskState) -> Self {
        match value {
            TaskState::New => "new".to_string(),
            TaskState::InProgress => "in_progress".to_string(),
            TaskState::Panicked => "panicked".to_string(),
            TaskState::Retry => "retry".to_string(),
            TaskState::Cancelled => "cancelled".to_string(),
            TaskState::Error => "error".to_string(),
            TaskState::Complete => "complete".to_string(),
            TaskState::TimedOut => "timed_out".to_string(),
            TaskState::Dead => "dead".to_string(),
        }
    }
}
