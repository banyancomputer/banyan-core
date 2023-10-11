#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        }
    }
}
